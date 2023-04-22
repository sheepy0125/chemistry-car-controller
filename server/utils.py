"""
Helper utilities
Created by sheepy0125 | MIT License | 2021-10-02
"""

### Classes ###


class Logger:
    """Log messages with ease"""

    colors: dict = {
        "log": "\033[92m",
        "warn": "\033[93m",
        "fatal": "\033[91m",
        "verbose": "\033[94m",
        "normal": "\033[0m",
    }

    @staticmethod
    def info(message: str):
        print(f"{Logger.colors['log']}[INFO] {message}{Logger.colors['normal']}")

    @staticmethod
    def warn(message: str):
        print(f"{Logger.colors['warn']}[WARN] {message}{Logger.colors['normal']}")

    @staticmethod
    def fatal(message: str):
        print(f"{Logger.colors['fatal']}[FAIL] {message}{Logger.colors['normal']}")

    @staticmethod
    def verbose(message: str):
        from shared import VERBOSE

        if not VERBOSE:
            return

        print(f"{Logger.colors['verbose']}[VERB] {message}{Logger.colors['normal']}")

    @staticmethod
    def log_error(error: Exception):
        error_type = type(error).__name__
        error_lines = []
        current_frame = error.__traceback__
        while True:
            try:
                error_lines.append(current_frame.tb_lineno)
                current_frame = current_frame.tb_next
                if current_frame is None:
                    break
            except Exception as _:
                break
        del current_frame
        if len(error_lines) == 0:
            error_lines.append("<unknown error lines>")
        try:
            error_file = str(error.__traceback__.tb_next.tb_frame).split("'")[1]
        except Exception as _:
            error_file = "<unknown file>"
        Logger.fatal(
            f"{error_type}: {error!s} (lines {', '.join(str(error_line) for error_line in reversed(error_lines))} "
            f"in file {error_file})"
        )
