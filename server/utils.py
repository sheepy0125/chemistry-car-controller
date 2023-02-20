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
        Logger.fatal(
            f"{type(error).__name__}: {str(error)} (line {error.__traceback__.tb_lineno})"
        )
