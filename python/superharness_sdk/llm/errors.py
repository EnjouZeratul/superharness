"""
LLM Errors

Error types for LLM client operations.
"""

from typing import Optional


class LlmError(Exception):
    """
    Base error for LLM operations.

    All LLM-related errors inherit from this class.
    """

    def __init__(self, message: str, provider: Optional[str] = None):
        super().__init__(message)
        self.provider = provider

    def __str__(self) -> str:
        if self.provider:
            return f"[{self.provider}] {super().__str__()}"
        return super().__str__()


class AuthenticationError(LlmError):
    """
    Raised when API authentication fails.

    Common causes:
        - Invalid API key
        - Expired API key
        - Wrong API key for provider
    """
    pass


class RateLimitError(LlmError):
    """
    Raised when rate limit is exceeded.

    Attributes:
        retry_after: Seconds to wait before retry (if provided)
    """

    def __init__(
        self,
        message: str,
        provider: Optional[str] = None,
        retry_after: Optional[float] = None,
    ):
        super().__init__(message, provider)
        self.retry_after = retry_after


class NetworkError(LlmError):
    """
    Raised when network connection fails.

    Common causes:
        - DNS resolution failure
        - Connection refused
        - SSL/TLS errors
    """
    pass


class TimeoutError(LlmError):
    """
    Raised when request times out.

    Attributes:
        timeout: The timeout duration in seconds
    """

    def __init__(
        self,
        message: str,
        provider: Optional[str] = None,
        timeout: Optional[float] = None,
    ):
        super().__init__(message, provider)
        self.timeout = timeout


class InvalidResponseError(LlmError):
    """
    Raised when API returns invalid response.

    Common causes:
        - Malformed JSON
        - Missing required fields
        - Unexpected response format
    """

    def __init__(
        self,
        message: str,
        provider: Optional[str] = None,
        response_data: Optional[dict] = None,
    ):
        super().__init__(message, provider)
        self.response_data = response_data


class ContentFilterError(LlmError):
    """
    Raised when content is filtered by provider safety systems.

    Common causes:
        - Harmful content detected
        - Policy violation
    """

    def __init__(
        self,
        message: str,
        provider: Optional[str] = None,
        filter_reason: Optional[str] = None,
    ):
        super().__init__(message, provider)
        self.filter_reason = filter_reason


class ModelNotFoundError(LlmError):
    """
    Raised when specified model is not available.
    """
    pass


class InsufficientQuotaError(LlmError):
    """
    Raised when account quota is exceeded.
    """
    pass


def classify_http_error(
    status_code: int,
    response_body: str,
    provider: str,
) -> LlmError:
    """
    Classify HTTP error into appropriate LlmError subclass.

    Args:
        status_code: HTTP status code
        response_body: Response body text
        provider: Provider name

    Returns:
        Appropriate LlmError subclass instance
    """
    if status_code == 401:
        return AuthenticationError(
            "Authentication failed. Check your API key.",
            provider=provider,
        )
    elif status_code == 403:
        return AuthenticationError(
            "Access forbidden. Check API key permissions.",
            provider=provider,
        )
    elif status_code == 404:
        return ModelNotFoundError(
            "Model or endpoint not found.",
            provider=provider,
        )
    elif status_code == 429:
        return RateLimitError(
            "Rate limit exceeded. Please wait and retry.",
            provider=provider,
        )
    elif status_code == 500:
        return LlmError(
            f"Server error: {response_body}",
            provider=provider,
        )
    elif status_code == 502:
        return NetworkError(
            "Bad gateway. Provider may be experiencing issues.",
            provider=provider,
        )
    elif status_code == 503:
        return NetworkError(
            "Service unavailable. Provider may be down.",
            provider=provider,
        )
    elif status_code == 504:
        return TimeoutError(
            "Gateway timeout. Request took too long.",
            provider=provider,
        )
    else:
        return LlmError(
            f"HTTP {status_code}: {response_body}",
            provider=provider,
        )
