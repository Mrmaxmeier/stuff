import warnings
warnings.warn("numba stub loaded")


def noop_decorator(func=None, *_args, **_kwargs):
    if callable(func):
        return func
    return lambda f: f


jit = njit = stencil = guvectorize = vectorize = cfunc = noop_decorator


__all__ = ["jit", "njit", "stencil", "guvectorize", "vectorize", "cfunc"]
