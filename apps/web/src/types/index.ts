export type DecisionType = "ALLOW" | "REVIEW" | "ESCALATE" | "BLOCK";

export interface StoredDecision {
  id: string;
  event_id: string;
  transaction_id: string;
  user_id: string;
  amount_minor: number;
  currency: string;
  channel: string;
  event_type: string;
  decision: DecisionType;
  reasons: string[];
  risk_score_bps?: number;
  is_fraud_predicted?: boolean;
  model_version?: string;
  features_snapshot: Record<string, number>;
  metadata: Record<string, string>;
  audit_id: string;
  created_at: string;
}

export interface DecisionStats {
  total_count: number;
  allow_count: number;
  review_count: number;
  escalate_count: number;
  block_count: number;
  total_volume_minor: number;
  block_rate_pct: number;
  review_queue_count: number;
  active_rules_count: number;
  ml_model_accuracy: number;
}

export interface AuditEvent {
  audit_id: string;
  event_id: string;
  decision: DecisionType;
  reasons: string[];
  feature_versions: Record<string, string>;
  rule_versions: Record<string, string>;
  model_version?: string | null;
  created_at: string;
}

export interface DecisionOutcome {
  decision: DecisionType;
  reasons: string[];
  audit_event: {
    audit_id: string;
    event_id: string;
    decision: DecisionType;
    reasons: string[];
    feature_versions: Record<string, string>;
    rule_versions: Record<string, string>;
    model_version?: string;
    created_at: string;
  };
  features_snapshot: Record<string, number>;
}

export interface RawTransactionPayload {
  event_id?: string;
  transaction_id: string;
  user_id: string;
  event_type: string;
  channel: string;
  amount_minor: number;
  currency: string;
  timestamp?: string;
  metadata?: Record<string, string>;
}

export interface ModelInfo {
  model_name: string;
  version: string;
  runtime: string;
  accuracy: string;
  precision: string;
  recall: string;
  f1_score: string;
  latency_p99_us: number;
  arbitration_strategy: string;
  features_catalog: Array<{
    name: string;
    type: string;
    window: string;
    description: string;
  }>;
  thresholds: {
    fraud_cutoff_bps: number;
    review_cutoff_bps: number;
  };
}

export interface AuditVerificationResult {
  valid: boolean;
  chain_status: "VERIFIED_TAMPER_EVIDENT" | "INTEGRITY_VIOLATION" | "INTEGRITY_COMPROMISED";
  verified_at: string;
  error?: string;
}

export interface UserProfile {
  id: string;
  name: string;
  email: string;
  role: string;
  department: string;
  permissions: string[];
}
