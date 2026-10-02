"""
AI service — converts natural language descriptions into structured strategy rules.

Uses OpenAI function calling to guarantee valid JSON output.
Falls back gracefully if API key is not configured.
"""

import json
from app.core.config import get_settings
from app.core.exceptions import ExternalServiceError

settings = get_settings()


SYSTEM_PROMPT = """You are EdgeArena's strategy builder assistant. You convert natural language
descriptions of trading strategies into structured JSON rules.

RULES SCHEMA:
Each strategy has:
- name: string
- asset: one of [BTC/USDT, ETH/USDT, SOL/USDT, BNB/USDT, XRP/USDT, ADA/USDT, DOGE/USDT, AVAX/USDT, DOT/USDT, MATIC/USDT, LINK/USDT, UNI/USDT, ATOM/USDT, LTC/USDT, NEAR/USDT, ARB/USDT, OP/USDT, INJ/USDT, SUI/USDT, APT/USDT]
- timeframe: one of [1m, 5m, 15m, 30m, 1h, 4h, 1d, 1w]
- entry_rules: array of conditions (ALL must be true — AND logic)
- exit_rules: array of conditions (ANY true triggers exit — OR logic)
- risk_management: {stop_loss_pct, take_profit_pct, position_size_pct, max_open_positions}

INDICATORS available:
- EMA(period), SMA(period), RMA(period)
- RSI(period)
- MACD(fast, slow, signal)
- Bollinger(period, std_dev)
- ATR(period)
- Stochastic(k_period, d_period)
- Volume, MFI(period), OBV

OPERATORS: >, <, >=, <=, ==, crosses_above, crosses_below, between, not_between

TARGET format:
- Static value: {"value": number}
- Compared to another indicator: {"indicator": "NAME", "params": {...}}

EXAMPLE:
User says: "Buy when the 9 EMA crosses above the 21 EMA on BTC 1h chart, with 2% stop loss and 4% take profit"
You output:
{
  "name": "EMA 9/21 Crossover",
  "asset": "BTC/USDT",
  "timeframe": "1h",
  "entry_rules": [
    {
      "type": "condition",
      "indicator": "EMA",
      "params": {"period": 9},
      "operator": "crosses_above",
      "target": {"indicator": "EMA", "params": {"period": 21}}
    }
  ],
  "exit_rules": [
    {
      "type": "condition",
      "indicator": "EMA",
      "params": {"period": 9},
      "operator": "crosses_below",
      "target": {"indicator": "EMA", "params": {"period": 21}}
    }
  ],
  "risk_management": {
    "stop_loss_pct": 2.0,
    "take_profit_pct": 4.0,
    "position_size_pct": 5.0,
    "max_open_positions": 1
  }
}

IMPORTANT:
- Always set "version": "1.0" in the rules output
- Always set "type": "condition" in each rule
- Use reasonable defaults for risk management if user doesn't specify
- Default position_size_pct: 5.0, max_open_positions: 1
- If user says "no stop loss", set stop_loss_pct to 0
- Be precise with indicator parameters. Use standard values (EMA 9/21, RSI 14, MACD 12/26/9, Bollinger 20/2)
"""

FUNCTION_DEF = {
    "name": "create_strategy",
    "description": "Create a structured trading strategy from a natural language description.",
    "parameters": {
        "type": "object",
        "properties": {
            "rules": {
                "type": "object",
                "description": "The complete strategy rules object.",
                "properties": {
                    "version": {"type": "string"},
                    "name": {"type": "string"},
                    "asset": {"type": "string"},
                    "timeframe": {"type": "string"},
                    "entry_rules": {"type": "array"},
                    "exit_rules": {"type": "array"},
                    "risk_management": {
                        "type": "object",
                        "properties": {
                            "stop_loss_pct": {"type": "number"},
                            "take_profit_pct": {"type": "number"},
                            "position_size_pct": {"type": "number"},
                            "max_open_positions": {"type": "integer"},
                        },
                    },
                },
                "required": ["version", "name", "asset", "timeframe", "entry_rules", "risk_management"],
            },
            "explanation": {
                "type": "string",
                "description": "A brief explanation of the strategy logic in plain English.",
            },
        },
        "required": ["rules", "explanation"],
    },
}


async def natural_language_to_rules(description: str) -> dict:
    """
    Convert a natural language description into structured strategy rules.

    Returns: {"rules": {...}, "explanation": "..."}
    Raises: ExternalServiceError if AI is not configured or fails.
    """
    if not settings.OPENAI_API_KEY:
        raise ExternalServiceError(
            "OpenAI",
            "AI assistant not configured. Set OPENAI_API_KEY to use this feature."
        )

    try:
        from openai import AsyncOpenAI
        client = AsyncOpenAI(api_key=settings.OPENAI_API_KEY)

        response = await client.chat.completions.create(
            model=settings.OPENAI_MODEL,
            messages=[
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": description},
            ],
            tools=[{"type": "function", "function": FUNCTION_DEF}],
            tool_choice={"type": "function", "function": {"name": "create_strategy"}},
            temperature=0.2,
            max_tokens=1500,
        )

        tool_call = response.choices[0].message.tool_calls[0]
        result = json.loads(tool_call.function.arguments)

        # Ensure version
        if "rules" in result:
            result["rules"]["version"] = "1.0"

        return result

    except ExternalServiceError:
        raise
    except Exception as e:
        raise ExternalServiceError("OpenAI", f"AI conversion failed: {str(e)}")


async def critique_strategy(rules: dict) -> str:
    """
    Analyze a strategy and return critique/suggestions.
    """
    if not settings.OPENAI_API_KEY:
        raise ExternalServiceError("OpenAI", "AI assistant not configured.")

    try:
        from openai import AsyncOpenAI
        client = AsyncOpenAI(api_key=settings.OPENAI_API_KEY)

        response = await client.chat.completions.create(
            model=settings.OPENAI_MODEL,
            messages=[
                {"role": "system", "content": (
                    "You are a trading strategy analyst. Review the provided strategy rules "
                    "and give constructive feedback. Consider: risk/reward ratio, indicator "
                    "conflicts, market regime suitability, potential overfitting, and "
                    "suggestions for improvement. Be concise and actionable."
                )},
                {"role": "user", "content": f"Review this strategy:\n{json.dumps(rules, indent=2)}"},
            ],
            temperature=0.4,
            max_tokens=800,
        )

        return response.choices[0].message.content or "No analysis available."

    except Exception as e:
        raise ExternalServiceError("OpenAI", f"AI critique failed: {str(e)}")


async def explain_indicator(indicator_name: str) -> str:
    """
    Get a plain-English explanation of an indicator.
    """
    if not settings.OPENAI_API_KEY:
        raise ExternalServiceError("OpenAI", "AI assistant not configured.")

    try:
        from openai import AsyncOpenAI
        client = AsyncOpenAI(api_key=settings.OPENAI_API_KEY)

        response = await client.chat.completions.create(
            model=settings.OPENAI_MODEL,
            messages=[
                {"role": "system", "content": "Explain the given technical indicator in 2-3 sentences. Be clear and concise for a non-expert trader."},
                {"role": "user", "content": f"Explain {indicator_name}"},
            ],
            temperature=0.3,
            max_tokens=300,
        )

        return response.choices[0].message.content or "No explanation available."

    except Exception as e:
        raise ExternalServiceError("OpenAI", f"AI explanation failed: {str(e)}")
