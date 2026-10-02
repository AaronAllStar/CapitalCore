from dataclasses import dataclass, field
from datetime import datetime


@dataclass
class Trade:
    entry_time: datetime
    entry_price: float
    quantity: float
    side: str
    stop_loss: float | None = None
    take_profit: float | None = None
    exit_time: datetime | None = None
    exit_price: float | None = None
    pnl: float = 0.0
    pnl_pct: float = 0.0


@dataclass
class Portfolio:
    initial_capital: float
    cash: float = 0.0
    positions: list[Trade] = field(default_factory=list)
    closed: list[Trade] = field(default_factory=list)
    equity: list[dict] = field(default_factory=list)

    def __post_init__(self):
        self.cash = self.initial_capital

    @property
    def total_equity(self) -> float:
        return self.cash

    def open(self, ts: datetime, price: float, size_pct: float, sl_pct: float | None, tp_pct: float | None) -> Trade | None:
        if self.positions:
            return None
        value = self.cash * (size_pct / 100)
        if value < 1:
            return None
        qty = value / price
        self.cash -= value
        trade = Trade(
            entry_time=ts, entry_price=price, quantity=qty, side="long",
            stop_loss=price * (1 - sl_pct / 100) if sl_pct else None,
            take_profit=price * (1 + tp_pct / 100) if tp_pct else None,
        )
        self.positions.append(trade)
        return trade

    def close(self, ts: datetime, price: float) -> Trade | None:
        if not self.positions:
            return None
        trade = self.positions.pop(0)
        trade.exit_time = ts
        trade.exit_price = price
        value = trade.quantity * price
        trade.pnl = value - (trade.quantity * trade.entry_price)
        trade.pnl_pct = (price / trade.entry_price - 1) * 100
        self.cash += value
        self.closed.append(trade)
        return trade

    def check_sl_tp(self, ts: datetime, low: float, high: float):
        for t in list(self.positions):
            if t.stop_loss and low <= t.stop_loss:
                self.close(ts, t.stop_loss)
            elif t.take_profit and high >= t.take_profit:
                self.close(ts, t.take_profit)

    def record(self, ts: datetime):
        self.equity.append({"timestamp": ts.isoformat(), "equity": round(self.total_equity, 2)})
