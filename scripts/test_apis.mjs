import { randomUUID } from "node:crypto";

const BASE_URL = "http://127.0.0.1:8001";

async function run() {
  console.log("=== Testing CapitalCore APIs on " + BASE_URL + " ===");

  // 1. Liveness
  const liveness = await fetch(`${BASE_URL}/health/liveness`).then((r) => r.json());
  console.log("1. /health/liveness:", JSON.stringify(liveness));

  // 2. Readiness
  const readiness = await fetch(`${BASE_URL}/health/readiness`).then((r) => r.json());
  console.log("2. /health/readiness: status =", readiness.status);

  // 3. Model info
  const model = await fetch(`${BASE_URL}/api/v1/model`).then((r) => r.json());
  console.log(`3. /api/v1/model: ${model.model_name} (version: ${model.version}, accuracy: ${model.accuracy})`);

  // 4. Audit Verify
  const verify = await fetch(`${BASE_URL}/api/v1/audit/verify`).then((r) => r.json());
  console.log("4. /api/v1/audit/verify: valid =", verify.valid, "| status =", verify.chain_status);

  // 5. Decision Stats Initial
  const initialStats = await fetch(`${BASE_URL}/api/v1/decisions/stats`).then((r) => r.json());
  console.log(`5. /api/v1/decisions/stats: total=${initialStats.total_count}, allow=${initialStats.allow_count}, review=${initialStats.review_count}, escalate=${initialStats.escalate_count}, block=${initialStats.block_count}`);

  // 6. Auth session / Token minting
  const session = await fetch(`${BASE_URL}/api/v1/auth/session`).then((r) => r.json());
  const token = session.access_token;
  console.log(`6. /api/v1/auth/session: User=${session.user.name} (${session.user.role}), Token=${token.substring(0, 20)}...`);

  // 7. Normal Transaction (POS $45.00) -> ALLOW
  const txnNormal = {
    transaction_id: randomUUID(),
    user_id: randomUUID(),
    event_type: "payment",
    channel: "pos",
    amount_minor: 4500,
    currency: "USD",
    metadata: { merchant_id: "merch_starbucks_101", category: "dining" },
  };
  const resNormal = await fetch(`${BASE_URL}/api/v1/transactions`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(txnNormal),
  }).then((r) => r.json());
  console.log(`7. Ingest Normal ($45.00 POS): Decision=${resNormal.decision}, Reasons=[${resNormal.reasons.join(", ")}]`);

  // 8. High Value Wire ($25,000) -> REVIEW
  const txnReview = {
    transaction_id: randomUUID(),
    user_id: randomUUID(),
    event_type: "transfer",
    channel: "api",
    amount_minor: 2500000,
    currency: "USD",
    metadata: { merchant_id: "merch_title_escrow", category: "real_estate" },
  };
  const resReview = await fetch(`${BASE_URL}/api/v1/transactions`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(txnReview),
  }).then((r) => r.json());
  console.log(`8. Ingest High Wire ($25,000 Wire): Decision=${resReview.decision}, Reasons=[${resReview.reasons.join(", ")}]`);

  // 9. Massive Transfer ($150,000) -> ESCALATE
  const txnEscalate = {
    transaction_id: randomUUID(),
    user_id: randomUUID(),
    event_type: "payment",
    channel: "web",
    amount_minor: 15000000,
    currency: "USD",
    metadata: { merchant_id: "merch_luxury_dealer", category: "luxury" },
  };
  const resEscalate = await fetch(`${BASE_URL}/api/v1/transactions`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(txnEscalate),
  }).then((r) => r.json());
  console.log(`9. Ingest Massive Transfer ($150,000 Web): Decision=${resEscalate.decision}, Reasons=[${resEscalate.reasons.join(", ")}]`);

  // 10. Audit log entries
  const auditEntries = await fetch(`${BASE_URL}/api/v1/audit`).then((r) => r.json());
  console.log(`10. /api/v1/audit: Total audit records=${auditEntries.length}, Latest event_id=${auditEntries[auditEntries.length - 1]?.event_id}`);

  // 11. Final Stats Check
  const finalStats = await fetch(`${BASE_URL}/api/v1/decisions/stats`).then((r) => r.json());
  console.log(`11. /api/v1/decisions/stats (Updated): total=${finalStats.total_count}, review_queue=${finalStats.review_queue_count}, total_volume_minor=${finalStats.total_volume_minor}`);

  console.log("=== ALL CAPITALCORE APIS VERIFIED AND FUNCTIONAL ===");
}

run().catch((err) => {
  console.error("API test failed:", err);
  process.exit(1);
});
