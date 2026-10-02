"""
Strategy validator — deep validation of rules against the indicator registry.

Returns structured (errors, warnings) where errors block execution
and warnings are advisory.
"""

from app.modules.strategies.schemas.strategy_schema import StrategyRulesSchema
from app.modules.strategies.services.indicator_registry import (
    REGISTRY,
    VALID_INDICATORS,
    VALID_TIMEFRAMES,
    VALID_OPERATORS,
    VALID_ASSETS,
)


def validate_rules(rules: StrategyRulesSchema) -> tuple[list[str], list[str]]:
    """Deep-validate strategy rules. Returns (errors, warnings)."""
    errors: list[str] = []
    warnings: list[str] = []

    # ── Top-level checks ────────────────────────────────────
    if not rules.name or not rules.name.strip():
        errors.append("Strategy name is required")

    if rules.timeframe not in VALID_TIMEFRAMES:
        errors.append(f"Invalid timeframe '{rules.timeframe}'. Valid: {sorted(VALID_TIMEFRAMES)}")

    if rules.asset not in VALID_ASSETS:
        warnings.append(f"Asset '{rules.asset}' not in standard list. Backtest may fail if data unavailable.")

    # ── Entry rules ─────────────────────────────────────────
    if not rules.entry_rules:
        errors.append("At least one entry rule is required")

    for i, rule in enumerate(rules.entry_rules):
        prefix = f"Entry[{i + 1}]"
        _validate_condition(rule, prefix, errors, warnings)

    # ── Exit rules ──────────────────────────────────────────
    for i, rule in enumerate(rules.exit_rules):
        prefix = f"Exit[{i + 1}]"
        _validate_condition(rule, prefix, errors, warnings)

    if not rules.exit_rules:
        warnings.append(
            "No exit rules defined. Positions will only close via stop-loss/take-profit."
        )

    # ── Risk management ─────────────────────────────────────
    rm = rules.risk_management

    if rm.stop_loss_pct > 0 and rm.take_profit_pct > 0:
        if rm.stop_loss_pct > rm.take_profit_pct:
            warnings.append(
                f"Stop-loss ({rm.stop_loss_pct}%) > Take-profit ({rm.take_profit_pct}%). "
                "This creates a negative risk/reward ratio."
            )
        else:
            rr = rm.take_profit_pct / rm.stop_loss_pct
            if rr < 1.5:
                warnings.append(
                    f"Risk/Reward ratio is {rr:.1f}:1. Consider at least 2:1 for positive expectancy."
                )

    if rm.position_size_pct > 50:
        warnings.append(
            f"Position size {rm.position_size_pct}% is very high. "
            "A single bad trade could lose half your capital."
        )
    elif rm.position_size_pct > 25:
        warnings.append(
            f"Position size {rm.position_size_pct}% is aggressive. Consider 5-10% for safer risk."
        )

    if rm.max_open_positions > 10:
        warnings.append(f"Max {rm.max_open_positions} open positions is very high. May over-leverage.")

    # ── Cross-rule checks ───────────────────────────────────
    _check_noisy_signals(rules, warnings)
    _check_exit_covers_entry(rules, warnings)

    return errors, warnings


def _validate_condition(rule, prefix: str, errors: list[str], warnings: list[str]):
    """Validate a single condition rule."""
    # Indicator check
    indicator = rule.indicator
    if indicator not in VALID_INDICATORS:
        errors.append(f"{prefix}: Invalid indicator '{indicator}'. Valid: {sorted(VALID_INDICATORS)}")
        return

    # Operator check
    if rule.operator not in VALID_OPERATORS:
        errors.append(f"{prefix}: Invalid operator '{rule.operator}'. Valid: {sorted(VALID_OPERATORS)}")
        return

    # Target check
    target = rule.target
    has_value = "value" in target and target["value"] is not None
    has_indicator = "indicator" in target and target["indicator"] is not None

    if not has_value and not has_indicator:
        errors.append(f"{prefix}: Target must have either 'value' or 'indicator'")
        return

    if has_indicator:
        target_ind = target["indicator"]
        if target_ind not in VALID_INDICATORS:
            errors.append(f"{prefix}: Target indicator '{target_ind}' is invalid")

    # Param validation against registry
    spec = REGISTRY.get(indicator)
    if spec and spec.params:
        for param_spec in spec.params:
            val = rule.params.get(param_spec.name)
            if val is not None:
                if val < param_spec.min or val > param_spec.max:
                    warnings.append(
                        f"{prefix}: {param_spec.label}={val} is outside recommended range "
                        f"[{param_spec.min}, {param_spec.max}]"
                    )

    # Check for always-true or always-false conditions
    if rule.operator in ("between", "not_between"):
        if "value" in target and "value_upper" not in target:
            errors.append(f"{prefix}: 'between' operator requires 'value' (lower) and 'value_upper' (upper)")


def _check_noisy_signals(rules: StrategyRulesSchema, warnings: list[str]):
    """Warn if entry and exit use the same condition (flip-flop)."""
    for entry in rules.entry_rules:
        for exit_rule in rules.exit_rules:
            if (
                entry.indicator == exit_rule.indicator
                and entry.params == exit_rule.params
                and entry.operator.replace("crosses_above", "crosses_below")
                    == exit_rule.operator.replace("crosses_below", "crosses_above")
            ):
                warnings.append(
                    "Entry and exit rules appear to be mirror images. "
                    "This creates rapid flip-flopping with high transaction costs."
                )
                return


def _check_exit_covers_entry(rules: StrategyRulesSchema, warnings: list[str]):
    """Warn if there's no way to exit a position except SL/TP."""
    if not rules.exit_rules:
        rm = rules.risk_management
        if rm.stop_loss_pct == 0 and rm.take_profit_pct == 0:
            warnings.append(
                "No exit rules AND no stop-loss/take-profit. "
                "Positions will never close — this is invalid."
            )
