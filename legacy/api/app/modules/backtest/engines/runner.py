import time
import pandas as pd
from app.modules.backtest.engines.indicators import compute_all
from app.modules.backtest.engines.signals import check_entry, check_exit
from app.modules.backtest.engines.portfolio import Portfolio
from app.modules.backtest.engines.metrics import calculate_metrics


def run_backtest(df: pd.DataFrame, rules: dict, initial_capital: float = 10000) -> dict:
    """Execute backtest on historical data. Returns results dict."""
    start = time.monotonic()

    if len(df) < 50:
        raise ValueError("Minimum 50 candles required")

    df = df.sort_values("timestamp").reset_index(drop=True)
    indicators = compute_all(df, rules)
    risk = rules.get("risk_management", {})

    portfolio = Portfolio(initial_capital=initial_capital)
    warmup = 50

    for i in range(warmup, len(df)):
        row = df.iloc[i]
        ts = row["timestamp"]

        portfolio.check_sl_tp(ts, float(row["low"]), float(row["high"]))

        if portfolio.positions and check_exit(rules, indicators, df, i):
            portfolio.close(ts, float(row["close"]))

        if not portfolio.positions and check_entry(rules, indicators, df, i):
            portfolio.open(
                ts, float(row["close"]),
                risk.get("position_size_pct", 5),
                risk.get("stop_loss_pct", 0) or None,
                risk.get("take_profit_pct", 0) or None,
            )

        portfolio.record(ts)

    # Close remaining
    if portfolio.positions:
        last = df.iloc[-1]
        portfolio.close(last["timestamp"], float(last["close"]))

    metrics = calculate_metrics(portfolio.closed, initial_capital, portfolio.equity)
    elapsed = int((time.monotonic() - start) * 1000)

    trades_data = []
    for t in portfolio.closed:
        trades_data.append({
            "entry_time": str(t.entry_time),
            "exit_time": str(t.exit_time) if t.exit_time else None,
            "side": t.side,
            "entry_price": round(t.entry_price, 8),
            "exit_price": round(t.exit_price, 8) if t.exit_price else None,
            "quantity": round(t.quantity, 8),
            "pnl": round(t.pnl, 4),
            "pnl_pct": round(t.pnl_pct, 4),
        })

    return {
        "results": metrics,
        "equity_curve": portfolio.equity,
        "trades": trades_data,
        "execution_time_ms": elapsed,
        "candle_count": len(df),
    }
