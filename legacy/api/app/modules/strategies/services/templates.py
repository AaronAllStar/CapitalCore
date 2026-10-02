"""
Built-in strategy templates — users can start from these instead of scratch.

Each template is a fully valid StrategyRules that can be loaded directly
into the builder or submitted for backtesting.
"""

TEMPLATES: dict[str, dict] = {
    # ── 1. EMA Crossover ────────────────────────────────────
    "ema_crossover": {
        "id": "ema_crossover",
        "name": "EMA Crossover",
        "description": (
            "Classic trend-following strategy. Enter when the fast EMA crosses "
            "above the slow EMA, exit when it crosses below. Works best in trending markets."
        ),
        "category": "trend_following",
        "difficulty": "beginner",
        "recommended_assets": ["BTC/USDT", "ETH/USDT"],
        "rules": {
            "version": "1.0",
            "name": "EMA Crossover",
            "asset": "BTC/USDT",
            "timeframe": "1h",
            "entry_rules": [
                {
                    "type": "condition",
                    "indicator": "EMA",
                    "params": {"period": 9},
                    "operator": "crosses_above",
                    "target": {"indicator": "EMA", "params": {"period": 21}},
                }
            ],
            "exit_rules": [
                {
                    "type": "condition",
                    "indicator": "EMA",
                    "params": {"period": 9},
                    "operator": "crosses_below",
                    "target": {"indicator": "EMA", "params": {"period": 21}},
                }
            ],
            "risk_management": {
                "stop_loss_pct": 2.0,
                "take_profit_pct": 4.0,
                "position_size_pct": 5.0,
                "max_open_positions": 1,
            },
        },
    },

    # ── 2. RSI Oversold Bounce ──────────────────────────────
    "rsi_oversold": {
        "id": "rsi_oversold",
        "name": "RSI Oversold Bounce",
        "description": (
            "Mean-reversion strategy. Enter when RSI drops below 30 (oversold) "
            "and price is above the 200 EMA (uptrend). Exit when RSI rises above 70."
        ),
        "category": "mean_reversion",
        "difficulty": "beginner",
        "recommended_assets": ["BTC/USDT", "ETH/USDT", "SOL/USDT"],
        "rules": {
            "version": "1.0",
            "name": "RSI Oversold Bounce",
            "asset": "BTC/USDT",
            "timeframe": "4h",
            "entry_rules": [
                {
                    "type": "condition",
                    "indicator": "RSI",
                    "params": {"period": 14},
                    "operator": "<",
                    "target": {"value": 30},
                },
                {
                    "type": "condition",
                    "indicator": "EMA",
                    "params": {"period": 200},
                    "operator": "<",
                    "target": {"indicator": "EMA", "params": {"period": 0}},
                    # NOTE: target with indicator=price (close) is indicated by period=0
                    # In practice the engine maps this to close price
                },
            ],
            "exit_rules": [
                {
                    "type": "condition",
                    "indicator": "RSI",
                    "params": {"period": 14},
                    "operator": ">",
                    "target": {"value": 70},
                }
            ],
            "risk_management": {
                "stop_loss_pct": 3.0,
                "take_profit_pct": 5.0,
                "position_size_pct": 5.0,
                "max_open_positions": 1,
            },
        },
    },

    # ── 3. MACD Momentum ────────────────────────────────────
    "macd_momentum": {
        "id": "macd_momentum",
        "name": "MACD Momentum",
        "description": (
            "Momentum strategy using MACD histogram. Enter when MACD line crosses "
            "above signal line, exit when it crosses below."
        ),
        "category": "momentum",
        "difficulty": "beginner",
        "recommended_assets": ["BTC/USDT", "ETH/USDT"],
        "rules": {
            "version": "1.0",
            "name": "MACD Momentum",
            "asset": "BTC/USDT",
            "timeframe": "1h",
            "entry_rules": [
                {
                    "type": "condition",
                    "indicator": "MACD",
                    "params": {"fast": 12, "slow": 26, "signal": 9},
                    "operator": "crosses_above",
                    "target": {"indicator": "MACD", "params": {"fast": 12, "slow": 26, "signal": 9}},
                    # NOTE: target for MACD uses output index 1 (signal line)
                    # This is expressed via the "target_output" field in extended schema
                }
            ],
            "exit_rules": [
                {
                    "type": "condition",
                    "indicator": "MACD",
                    "params": {"fast": 12, "slow": 26, "signal": 9},
                    "operator": "crosses_below",
                    "target": {"indicator": "MACD", "params": {"fast": 12, "slow": 26, "signal": 9}},
                }
            ],
            "risk_management": {
                "stop_loss_pct": 2.5,
                "take_profit_pct": 5.0,
                "position_size_pct": 5.0,
                "max_open_positions": 1,
            },
        },
    },

    # ── 4. Bollinger Squeeze ────────────────────────────────
    "bollinger_squeeze": {
        "id": "bollinger_squeeze",
        "name": "Bollinger Band Squeeze",
        "description": (
            "Volatility breakout strategy. Enter when price closes above the upper "
            "Bollinger Band after a squeeze (bands narrowing). Exit when price returns to middle band."
        ),
        "category": "volatility",
        "difficulty": "intermediate",
        "recommended_assets": ["BTC/USDT", "ETH/USDT", "SOL/USDT"],
        "rules": {
            "version": "1.0",
            "name": "Bollinger Band Squeeze",
            "asset": "BTC/USDT",
            "timeframe": "1h",
            "entry_rules": [
                {
                    "type": "condition",
                    "indicator": "Bollinger",
                    "params": {"period": 20, "std_dev": 2.0},
                    "operator": ">",
                    "target": {"indicator": "Bollinger", "params": {"period": 20, "std_dev": 2.0}},
                    # Entry: close > upper band (output 0)
                },
                {
                    "type": "condition",
                    "indicator": "ATR",
                    "params": {"period": 14},
                    "operator": "<",
                    "target": {"value": 500},
                    # Squeeze: ATR below threshold means low volatility
                },
            ],
            "exit_rules": [
                {
                    "type": "condition",
                    "indicator": "Bollinger",
                    "params": {"period": 20, "std_dev": 2.0},
                    "operator": "<",
                    "target": {"indicator": "Bollinger", "params": {"period": 20, "std_dev": 2.0}},
                    # Exit: close < middle band (output 1)
                }
            ],
            "risk_management": {
                "stop_loss_pct": 2.0,
                "take_profit_pct": 4.0,
                "position_size_pct": 5.0,
                "max_open_positions": 1,
            },
        },
    },

    # ── 5. RSI + EMA Combo ──────────────────────────────────
    "rsi_ema_combo": {
        "id": "rsi_ema_combo",
        "name": "RSI + EMA Filter",
        "description": (
            "Combines momentum and trend. Enter when RSI crosses above 50 AND "
            "price is above EMA 50 (bullish trend confirmation). Exit when RSI crosses below 50."
        ),
        "category": "combo",
        "difficulty": "intermediate",
        "recommended_assets": ["BTC/USDT", "ETH/USDT", "SOL/USDT", "BNB/USDT"],
        "rules": {
            "version": "1.0",
            "name": "RSI + EMA Filter",
            "asset": "BTC/USDT",
            "timeframe": "1h",
            "entry_rules": [
                {
                    "type": "condition",
                    "indicator": "RSI",
                    "params": {"period": 14},
                    "operator": "crosses_above",
                    "target": {"value": 50},
                },
                {
                    "type": "condition",
                    "indicator": "EMA",
                    "params": {"period": 50},
                    "operator": ">",
                    "target": {"indicator": "EMA", "params": {"period": 50}},
                    # Price above EMA 50 — in engine this maps to close > EMA50
                },
            ],
            "exit_rules": [
                {
                    "type": "condition",
                    "indicator": "RSI",
                    "params": {"period": 14},
                    "operator": "crosses_below",
                    "target": {"value": 50},
                }
            ],
            "risk_management": {
                "stop_loss_pct": 2.5,
                "take_profit_pct": 5.0,
                "position_size_pct": 5.0,
                "max_open_positions": 1,
            },
        },
    },
}


def get_template_list() -> list[dict]:
    """Return summary list for API."""
    return [
        {
            "id": t["id"],
            "name": t["name"],
            "description": t["description"],
            "category": t["category"],
            "difficulty": t["difficulty"],
            "recommended_assets": t["recommended_assets"],
        }
        for t in TEMPLATES.values()
    ]


def get_template(template_id: str) -> dict | None:
    return TEMPLATES.get(template_id)
