import pytest
import pandas as pd
import numpy as np
from datetime import datetime, timedelta
from app.modules.backtest.engines.runner import run_backtest
from app.modules.backtest.engines.indicators import ema, rsi, bollinger
from app.modules.strategies.services.validator import validate_rules
from app.modules.strategies.schemas.strategy_schema import StrategyRulesSchema


def make_data(n=500):
    np.random.seed(42)
    dates = [datetime(2024, 1, 1) + timedelta(hours=i) for i in range(n)]
    base = 40000
    prices = base + np.cumsum(np.random.randn(n) * 100)
    return pd.DataFrame({
        "timestamp": dates,
        "open": prices - np.random.rand(n) * 50,
        "high": prices + np.random.rand(n) * 100,
        "low": prices - np.random.rand(n) * 100,
        "close": prices,
        "volume": np.random.rand(n) * 1e6,
    })


STRATEGY = {
    "version": "1.0",
    "name": "EMA Cross",
    "asset": "BTC/USDT",
    "timeframe": "1h",
    "entry_rules": [{
        "type": "condition",
        "indicator": "EMA",
        "params": {"period": 9},
        "operator": "crosses_above",
        "target": {"indicator": "EMA", "params": {"period": 21}},
    }],
    "exit_rules": [{
        "type": "condition",
        "indicator": "EMA",
        "params": {"period": 9},
        "operator": "crosses_below",
        "target": {"indicator": "EMA", "params": {"period": 21}},
    }],
    "risk_management": {
        "stop_loss_pct": 2.0,
        "take_profit_pct": 4.0,
        "position_size_pct": 10.0,
        "max_open_positions": 1,
    },
}


class TestIndicators:
    def test_ema(self):
        s = pd.Series(range(1, 11), dtype=float)
        result = ema(s, 3)
        assert len(result) == 10
        assert not pd.isna(result.iloc[-1])

    def test_rsi(self):
        s = pd.Series(np.random.rand(50) * 100)
        result = rsi(s, 14)
        assert 0 <= result.dropna().iloc[-1] <= 100


class TestBacktest:
    def test_basic(self):
        df = make_data(500)
        result = run_backtest(df, STRATEGY)
        assert "results" in result
        assert result["results"]["total_trades"] >= 0
        assert "execution_time_ms" in result

    def test_too_few_candles(self):
        with pytest.raises(ValueError, match="Minimum"):
            run_backtest(make_data(30), STRATEGY)


class TestValidation:
    def test_valid(self):
        rules = StrategyRulesSchema(**STRATEGY)
        errors, warnings = validate_rules(rules)
        assert errors == []

    def test_invalid_timeframe(self):
        data = {**STRATEGY, "timeframe": "2h"}
        rules = StrategyRulesSchema(**data)
        errors, _ = validate_rules(rules)
        assert any("timeframe" in e.lower() for e in errors)

    def test_negative_rr_warning(self):
        data = {**STRATEGY, "risk_management": {
            "stop_loss_pct": 5.0, "take_profit_pct": 2.0,
            "position_size_pct": 10.0, "max_open_positions": 1,
        }}
        rules = StrategyRulesSchema(**data)
        _, warnings = validate_rules(rules)
        assert any("negative" in w.lower() for w in warnings)
