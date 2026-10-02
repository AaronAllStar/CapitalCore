"""
Indicator registry — defines every supported indicator with its parameters.

This is the single source of truth for:
  - Frontend: populate indicator dropdowns
  - Validator: check indicator/param validity
  - Backtest engine: map names to computation functions
  - AI service: include in prompt for structured output
"""

from dataclasses import dataclass, field


@dataclass
class ParamSpec:
    name: str
    label: str
    type: str = "int"        # int | float
    default: float = 20
    min: float = 1
    max: float = 500
    step: float = 1


@dataclass
class IndicatorSpec:
    name: str
    label: str
    description: str
    category: str                         # trend | momentum | volatility | volume
    params: list[ParamSpec] = field(default_factory=list)
    outputs: int = 1                      # 1 = single series, 2+ = multi (macd=3, bollinger=3, stoch=2)
    output_labels: list[str] = field(default_factory=list)


REGISTRY: dict[str, IndicatorSpec] = {}

def _register(spec: IndicatorSpec) -> IndicatorSpec:
    REGISTRY[spec.name] = spec
    return spec


# ── Trend ───────────────────────────────────────────────────
EMA = _register(IndicatorSpec(
    name="EMA", label="Exponential Moving Average",
    description="Weighted average giving more importance to recent prices.",
    category="trend",
    params=[ParamSpec("period", "Period", default=20, min=2, max=500)],
    output_labels=["EMA"],
))

SMA = _register(IndicatorSpec(
    name="SMA", label="Simple Moving Average",
    description="Arithmetic mean of closing prices over N periods.",
    category="trend",
    params=[ParamSpec("period", "Period", default=20, min=2, max=500)],
    output_labels=["SMA"],
))

RMA = _register(IndicatorSpec(
    name="RMA", label="Wilder's Moving Average",
    description="Smoothed moving average used in RSI and ADX.",
    category="trend",
    params=[ParamSpec("period", "Period", default=14, min=2, max=500)],
    output_labels=["RMA"],
))


# ── Momentum ────────────────────────────────────────────────
RSI = _register(IndicatorSpec(
    name="RSI", label="Relative Strength Index",
    description="Measures speed and magnitude of price changes. 0-100 scale.",
    category="momentum",
    params=[ParamSpec("period", "Period", default=14, min=2, max=100)],
    output_labels=["RSI"],
))

MACD = _register(IndicatorSpec(
    name="MACD", label="Moving Average Convergence Divergence",
    description="Difference between fast and slow EMA. Three outputs: line, signal, histogram.",
    category="momentum",
    params=[
        ParamSpec("fast", "Fast Period", default=12, min=2, max=100),
        ParamSpec("slow", "Slow Period", default=26, min=5, max=200),
        ParamSpec("signal", "Signal Period", default=9, min=2, max=50),
    ],
    outputs=3,
    output_labels=["MACD Line", "Signal Line", "Histogram"],
))

STOCHASTIC = _register(IndicatorSpec(
    name="Stochastic", label="Stochastic Oscillator",
    description="%K and %D lines comparing close to high/low range.",
    category="momentum",
    params=[
        ParamSpec("k_period", "K Period", default=14, min=2, max=100),
        ParamSpec("d_period", "D Period (smoothing)", default=3, min=1, max=20),
    ],
    outputs=2,
    output_labels=["%K", "%D"],
))


# ── Volatility ──────────────────────────────────────────────
BOLLINGER = _register(IndicatorSpec(
    name="Bollinger", label="Bollinger Bands",
    description="SMA ± N standard deviations. Three outputs: upper, middle, lower.",
    category="volatility",
    params=[
        ParamSpec("period", "Period", default=20, min=5, max=200),
        ParamSpec("std_dev", "Std Deviations", type="float", default=2.0, min=0.5, max=5.0, step=0.1),
    ],
    outputs=3,
    output_labels=["Upper Band", "Middle Band", "Lower Band"],
))

ATR = _register(IndicatorSpec(
    name="ATR", label="Average True Range",
    description="Measures market volatility based on true range.",
    category="volatility",
    params=[ParamSpec("period", "Period", default=14, min=2, max=100)],
    output_labels=["ATR"],
))


# ── Volume ──────────────────────────────────────────────────
VOLUME = _register(IndicatorSpec(
    name="Volume", label="Volume",
    description="Raw trading volume per candle.",
    category="volume",
    params=[],
    output_labels=["Volume"],
))

MFI = _register(IndicatorSpec(
    name="MFI", label="Money Flow Index",
    description="Volume-weighted RSI. 0-100 scale.",
    category="volume",
    params=[ParamSpec("period", "Period", default=14, min=2, max=100)],
    output_labels=["MFI"],
))

OBV = _register(IndicatorSpec(
    name="OBV", label="On-Balance Volume",
    description="Cumulative volume based on price direction.",
    category="volume",
    params=[],
    output_labels=["OBV"],
))


# ── Constants derived from registry ─────────────────────────
VALID_INDICATORS: set[str] = set(REGISTRY.keys())
INDICATOR_DEFAULTS: dict[str, dict[str, float]] = {
    name: {p.name: p.default for p in spec.params}
    for name, spec in REGISTRY.items()
}

VALID_TIMEFRAMES = {"1m", "5m", "15m", "30m", "1h", "4h", "1d", "1w"}

VALID_OPERATORS = {
    ">", "<", ">=", "<=", "==",
    "crosses_above", "crosses_below",
    "between", "not_between",
}

VALID_ASSETS = {
    "BTC/USDT", "ETH/USDT", "SOL/USDT", "BNB/USDT", "XRP/USDT",
    "ADA/USDT", "DOGE/USDT", "AVAX/USDT", "DOT/USDT", "MATIC/USDT",
    "LINK/USDT", "UNI/USDT", "ATOM/USDT", "LTC/USDT", "NEAR/USDT",
    "ARB/USDT", "OP/USDT", "INJ/USDT", "SUI/USDT", "APT/USDT",
}


def get_registry_json() -> list[dict]:
    """Serialize registry for API responses (frontend dropdowns)."""
    return [
        {
            "name": spec.name,
            "label": spec.label,
            "description": spec.description,
            "category": spec.category,
            "outputs": spec.outputs,
            "output_labels": spec.output_labels,
            "params": [
                {"name": p.name, "label": p.label, "type": p.type,
                 "default": p.default, "min": p.min, "max": p.max, "step": p.step}
                for p in spec.params
            ],
        }
        for spec in REGISTRY.values()
    ]
