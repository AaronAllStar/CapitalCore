import pandas as pd


def _get_val(indicator_result, idx: int) -> float | None:
    if isinstance(indicator_result, tuple):
        val = indicator_result[0].iloc[idx]
    else:
        val = indicator_result.iloc[idx]
    if pd.isna(val):
        return None
    return float(val)


def _get_target(rule: dict, indicators: dict, idx: int) -> float | None:
    target = rule.get("target", {})
    if "value" in target:
        return float(target["value"])
    if "indicator" in target:
        t_ind = indicators.get(target["indicator"])
        if t_ind is None:
            return None
        return _get_val(t_ind, idx)
    return None


def check_condition(rule: dict, indicators: dict, df: pd.DataFrame, idx: int) -> bool:
    ind = indicators.get(rule["indicator"])
    if ind is None:
        return False

    current = _get_val(ind, idx)
    if current is None:
        return False

    target = _get_target(rule, indicators, idx)
    if target is None:
        return False

    op = rule["operator"]

    if op == ">":
        return current > target
    elif op == "<":
        return current < target
    elif op == ">=":
        return current >= target
    elif op == "<=":
        return current <= target
    elif op == "==":
        return abs(current - target) < 1e-8
    elif op in ("crosses_above", "crosses_below"):
        if idx < 1:
            return False
        prev_val = _get_val(ind, idx - 1)
        prev_target = _get_target(rule, indicators, idx - 1)
        if prev_val is None or prev_target is None:
            return False
        if op == "crosses_above":
            return prev_val <= prev_target and current > target
        else:
            return prev_val >= prev_target and current < target

    return False


def check_entry(rules: dict, indicators: dict, df: pd.DataFrame, idx: int) -> bool:
    """All entry rules must be true (AND logic)."""
    entry_rules = rules.get("entry_rules", [])
    if not entry_rules:
        return False
    return all(check_condition(r, indicators, df, idx) for r in entry_rules)


def check_exit(rules: dict, indicators: dict, df: pd.DataFrame, idx: int) -> bool:
    """Any exit rule true triggers exit (OR logic)."""
    return any(check_condition(r, indicators, df, idx) for r in rules.get("exit_rules", []))
