from __future__ import annotations
from typing import Any
from fastapi import HTTPException, status


class AppException(HTTPException):
    """Base application exception with machine-readable code."""

    def __init__(
        self,
        status_code: int = 500,
        detail: str = "Internal server error",
        code: str = "INTERNAL_ERROR",
        headers: dict[str, str] | None = None,
    ):
        self.code = code
        super().__init__(status_code=status_code, detail=detail, headers=headers)


class NotFoundError(AppException):
    def __init__(self, resource: str = "Resource", resource_id: str | None = None):
        detail = f"{resource} not found"
        if resource_id:
            detail = f"{resource} '{resource_id}' not found"
        super().__init__(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=detail,
            code="NOT_FOUND",
        )


class UnauthorizedError(AppException):
    def __init__(self, detail: str = "Authentication required"):
        super().__init__(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=detail,
            code="UNAUTHORIZED",
            headers={"WWW-Authenticate": "Bearer"},
        )


class ForbiddenError(AppException):
    def __init__(self, detail: str = "Insufficient permissions"):
        super().__init__(
            status_code=status.HTTP_403_FORBIDDEN,
            detail=detail,
            code="FORBIDDEN",
        )


class ConflictError(AppException):
    def __init__(self, detail: str = "Resource conflict"):
        super().__init__(
            status_code=status.HTTP_409_CONFLICT,
            detail=detail,
            code="CONFLICT",
        )


class ValidationError(AppException):
    def __init__(self, detail: str = "Validation failed", errors: list[dict] | None = None):
        self.errors = errors or []
        super().__init__(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail=detail,
            code="VALIDATION_ERROR",
        )


class RateLimitError(AppException):
    def __init__(self, detail: str = "Rate limit exceeded", retry_after: int = 60):
        super().__init__(
            status_code=status.HTTP_429_TOO_MANY_REQUESTS,
            detail=detail,
            code="RATE_LIMITED",
            headers={"Retry-After": str(retry_after)},
        )


class PlanLimitError(AppException):
    def __init__(self, detail: str = "Plan limit reached", plan: str = "free"):
        super().__init__(
            status_code=status.HTTP_402_PAYMENT_REQUIRED,
            detail=detail,
            code="PLAN_LIMIT",
        )


class BusinessRuleError(AppException):
    def __init__(self, detail: str, code: str = "BUSINESS_RULE"):
        super().__init__(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=detail,
            code=code,
        )


class ExternalServiceError(AppException):
    def __init__(self, service: str, detail: str = "External service unavailable"):
        super().__init__(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail=f"{service}: {detail}",
            code="EXTERNAL_SERVICE_ERROR",
        )
