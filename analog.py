from collections.abc import Mapping, Sequence


def pydig(obj, *args, path=None, default=None):
    if args:
        for arg in args:
            if isinstance(arg, str) and hasattr(obj, arg):
                obj = getattr(obj, arg)
            elif isinstance(obj, Mapping) and arg in obj:
                obj = obj[arg]
            elif isinstance(obj, Sequence):
                try:
                    obj = obj[int(arg)]
                except:
                    return None
            else:
                return None
    elif path:
        for k in path.split("."):
            if hasattr(obj, k):
                obj = getattr(obj, k)
            elif isinstance(obj, Mapping) and k in obj:
                obj = obj[k]
            elif k.isdigit() and isinstance(obj, Sequence):
                try:
                    obj = obj[int(k)]
                except:
                    return None
            else:
                return None
    return obj
