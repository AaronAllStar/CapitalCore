import numpy as np
import pandas as pd


def ema(series: pd.Series, period: int) -> pd.Series:
    return series.ewm(span=period, adjust=False).mean()


def sma(series: pd.Series, period: int) -> pd.Series:
    return series.rolling(window=period).mean()


def rma(series: pd.Series, period: int) -> pd.Series:
    return series.ewm(alpha=1 / period, adjust=False).mean()


def rsi(series: pd.Series, period: int = 14) -> pd.Series:
    delta = series.diff()
    gain = delta.where(delta > 0, 0.0)
    loss = -delta.where(delta < 0, 0.0)
    avg_gain = rma(gain, period)
    avg_loss = rma(loss, period)
    rs = avg_gain / avg_loss.replace(0, np.nan)
    return 100 - (100 / (1 + rs))


def macd(series: pd.Series, fast: int = 12, slow: int = 26, signal_period: int = 9):
    fast_ema = ema(series, fast)
    slow_ema = ema(series, slow)
    macd_line = fast_ema - slow_ema
    signal_line = ema(macd_line, signal_period)
    histogram = macd_line - signal_line
    return macd_line, signal_line, histogram


def bollinger(series: pd.Series, period: int = 20, std_dev: float = 2.0):
    middle = sma(series, period)
    std = series.rolling(window=period).std()
    upper = middle + std_dev * std
    lower = middle - std_dev * std
    return upper, middle, lower


def atr(high: pd.Series, low: pd.Series, close: pd.Series, period: int = 14) -> pd.Series:
    tr = pd.concat([
        high - low,
        (high - close.shift(1)).abs(),
        (low - close.shift(1)).abs(),
    ], axis=1).max(axis=1)
    return rma(tr, period)


def stochastic_kd(high: pd.Series, low: pd.Series, close: pd.Series, k_period: int = 14, d_period: int = 3):
    lowest = low.rolling(window=k_period).min()
    highest = high.rolling(window=k_period).max()
    k = 100 * (close - lowest) / (highest - lowest).replace(0, np.nan)
    d = sma(k, d_period)
    return k, d


INDICATOR_MAP = {
    "EMA": lambda df, p: ema(df["close"], int(p.get("period", 20))),
    "SMA": lambda df, p: sma(df["close"], int(p.get("period", 20))),
    "RMA": lambda df, p: rma(df["close"], int(p.get("period", 20))),
    "RSI": lambda df, p: rsi(df["close"], int(p.get("period", 14))),
    "MACD": lambda df, p: macd(df["close"], int(p.get("fast", 12)), int(p.get("slow", 26)), int(p.get("signal", 9))),
    "Bollinger": lambda df, p: bollinger(df["close"], int(p.get("period", 20)), float(p.get("std_dev", 2.0))),
    "ATR": lambda df, p: atr(df["high"], df["low"], df["close"], int(p.get("period", 14))),
    "Stochastic": lambda df, p: stochastic_kd(df["high"], df["low"], df["close"], int(p.get("k_period", 14)), int(p.get("d_period", 3))),
    "Volume": lambda df, p: df["volume"],
}


def compute_all(df: pd.DataFrame, rules: dict) -> dict:
    indicators = {}
    all_rules = rules.get("entry_rules", []) + rules.get("exit_rules", [])
    needed = set()
    for rule in all_rules:
        needed.add(rule["indicator"])
        if "indicator" in rule.get("target", {}):
            needed.add(rule["target"]["indicator"])

    for name in needed:
        if name in INDICATOR_MAP:
            for rule in all_rules:
                if rule["indicator"] == name:
                    indicators[name] = INDICATOR_MAP[name](df, rule.get("params", {}))
                    break
    return indicators
