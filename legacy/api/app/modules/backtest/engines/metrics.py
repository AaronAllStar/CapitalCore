import numpy as np
from app.modules.backtest.engines.portfolio import Trade


def calculate_metrics(trades: list[Trade], initial_capital: float, equity: list[dict]) -> dict:
    if not trades:
        return {
            "pnl_pct": 0.0, "total_trades": 0, "win_rate": 0.0, "sharpe_ratio": 0.0,
            "max_drawdown_pct": 0.0, "profit_factor": 0.0, "avg_win": 0.0, "avg_loss": 0.0,
            "best_trade": 0.0, "worst_trade": 0.0, "avg_trade_duration_hours": 0.0,
        }

    pnls = [t.pnl for t in trades]
    pnl_pcts = [t.pnl_pct for t in trades]
    wins = [p for p in pnls if p > 0]
    losses = [p for p in pnls if p <= 0]

    total_pnl = sum(pnls)
    pnl_pct = (total_pnl / initial_capital) * 100
    win_rate = (len(wins) / len(trades)) * 100

    avg_win = float(np.mean(wins)) if wins else 0.0
    avg_loss = abs(float(np.mean(losses))) if losses else 0.0
    pf = (sum(wins) / abs(sum(losses))) if losses and sum(losses) != 0 else 0.0

    if len(pnl_pcts) > 1:
        arr = np.array(pnl_pcts)
        sharpe = float((np.mean(arr) / np.std(arr)) * np.sqrt(len(arr))) if np.std(arr) > 0 else 0.0
    else:
        sharpe = 0.0

    equities = [e["equity"] for e in equity]
    peak = equities[0] if equities else 0
    max_dd = 0.0
    for eq in equities:
        peak = max(peak, eq)
        dd = (peak - eq) / peak if peak > 0 else 0
        max_dd = max(max_dd, dd)

    # Avg trade duration
    durations = []
    for t in trades:
        if t.exit_time and t.entry_time:
            dur = (t.exit_time - t.entry_time).total_seconds() / 3600
            durations.append(dur)
    avg_dur = float(np.mean(durations)) if durations else 0.0

    return {
        "pnl_pct": round(pnl_pct, 4),
        "total_trades": len(trades),
        "win_rate": round(win_rate, 4),
        "sharpe_ratio": round(sharpe, 4),
        "max_drawdown_pct": round(max_dd * 100, 4),
        "profit_factor": round(pf, 4),
        "avg_win": round(avg_win, 4),
        "avg_loss": round(avg_loss, 4),
        "best_trade": round(max(pnl_pcts), 4) if pnl_pcts else 0.0,
        "worst_trade": round(min(pnl_pcts), 4) if pnl_pcts else 0.0,
        "avg_trade_duration_hours": round(avg_dur, 2),
    }
