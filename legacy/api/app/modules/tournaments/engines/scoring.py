"""Tournament scoring engine — calculates composite scores from backtest results."""


DEFAULT_SCORING = {
    "pnl_weight": 0.35,
    "sharpe_weight": 0.25,
    "drawdown_weight": 0.20,
    "winrate_weight": 0.10,
    "pf_weight": 0.10,
}


def normalize(values: list[float], higher_is_better: bool = True) -> list[float]:
    """Normalize values to 0-1 range."""
    if not values:
        return []
    mn, mx = min(values), max(values)
    if mx == mn:
        return [0.5] * len(values)
    normed = [(v - mn) / (mx - mn) for v in values]
    if not higher_is_better:
        normed = [1 - n for n in normed]
    return normed


def calculate_scores(
    results: list[dict],
    scoring_config: dict | None = None,
) -> list[dict]:
    """
    Calculate composite scores for a list of backtest results.

    Args:
        results: List of dicts with keys: pnl_pct, sharpe_ratio, max_drawdown_pct, win_rate, profit_factor
        scoring_config: Optional custom weights

    Returns:
        List of results with 'score' field added, sorted by score descending
    """
    if not results:
        return []

    config = {**DEFAULT_SCORING, **(scoring_config or {})}

    pnls = [r.get("pnl_pct", 0) for r in results]
    sharpes = [r.get("sharpe_ratio", 0) for r in results]
    drawdowns = [r.get("max_drawdown_pct", 100) for r in results]
    winrates = [r.get("win_rate", 0) for r in results]
    pfs = [r.get("profit_factor", 0) for r in results]

    norm_pnl = normalize(pnls, higher_is_better=True)
    norm_sharpe = normalize(sharpes, higher_is_better=True)
    norm_dd = normalize(drawdowns, higher_is_better=False)  # Lower drawdown is better
    norm_wr = normalize(winrates, higher_is_better=True)
    norm_pf = normalize(pfs, higher_is_better=True)

    scored = []
    for i, r in enumerate(results):
        score = (
            norm_pnl[i] * config["pnl_weight"]
            + norm_sharpe[i] * config["sharpe_weight"]
            + norm_dd[i] * config["drawdown_weight"]
            + norm_wr[i] * config["winrate_weight"]
            + norm_pf[i] * config["pf_weight"]
        )
        scored.append({**r, "score": round(score * 100, 4)})

    scored.sort(key=lambda x: x["score"], reverse=True)
    for i, s in enumerate(scored):
        s["rank"] = i + 1

    return scored
