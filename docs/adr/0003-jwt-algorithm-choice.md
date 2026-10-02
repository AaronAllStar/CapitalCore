# ADR-0003: JWT Algorithm Choice

## Context
The legacy Python backend utilized symmetric HS256 JWT tokens with a weak, hardcoded default secret key (`change-me-use-openssl-rand-hex-32`). In a distributed architecture where multiple microservices or edge instances need to verify tokens, sharing the symmetric HMAC secret with every service creates a severe security risk: any verifier can also forge valid signatures. Furthermore, [AGENTS.md](file:///c:/Users/damed/Downloads/edge-arena/docs/AGENTS.md) prioritizes `Security > Performance`.

## Decision
Adopt asymmetric **Ed25519 (EdDSA)** for all authentication JWT signing and verification:
- Token issuer (`edge-auth`) holds the private signing key in secure configuration.
- Verifying services (e.g. `edge-api` middleware) possess only the public key.
- Key rotation is supported via a standard `kid` (Key ID) header pointing to an in-memory or JWKS key ring.
- Standard JWT claims include `sub` (user UUID), `exp`, `iat`, `iss`, and `type` (access vs refresh).

## Alternatives Considered
1. **HS256 (HMAC-SHA256)**: Rejected because verifying nodes must share the signing secret, creating high exposure and secret sprawl.
2. **RS256 (RSA 2048/4096-bit)**: Viable industry standard, but Ed25519 produces significantly smaller signature sizes (64 bytes vs 256/512 bytes), provides faster cryptographic operations, and is resilient against side-channel and timing attacks.
3. **Paseto (Platform-Agnostic Security Tokens)**: High security, but lacks broad multi-platform client tooling compared to standard Ed25519 JWTs.

## Consequences
- **Positive**: Strict separation of signing privilege from verification privilege; compact header/payload overhead; exceptional verification performance; resilient cryptographic primitives.
- **Negative / Risks**: Key generation and public-key distribution requires structured operational handling; existing legacy HS256 sessions cannot be migrated and will require re-authentication.

## Test Strategy
- Unit tests: verify signing with private key and verification with public key.
- Negative tests: ensure verification strictly fails when given tampered claims, expired timestamps, wrong algorithm headers (`alg: none` rejection), or invalid public keys.
- Property testing: token round-tripping with random claims.

## Migration Impact
- Legacy HS256 tokens will be invalidated upon cutover; users will re-authenticate against the new Ed25519 auth endpoint.
