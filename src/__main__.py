"""CLI for devops-cli."""
import sys, json, argparse
from .core import DevopsCli

def main():
    parser = argparse.ArgumentParser(description="Rust CLI DevOps toolkit with 11 subcommands for port scanning, HTTP, JSON, Docker, and more")
    parser.add_argument("command", nargs="?", default="status", choices=["status", "run", "info"])
    parser.add_argument("--input", "-i", default="")
    args = parser.parse_args()
    instance = DevopsCli()
    if args.command == "status":
        print(json.dumps(instance.get_stats(), indent=2))
    elif args.command == "run":
        print(json.dumps(instance.detect(input=args.input or "test"), indent=2, default=str))
    elif args.command == "info":
        print(f"devops-cli v0.1.0 — Rust CLI DevOps toolkit with 11 subcommands for port scanning, HTTP, JSON, Docker, and more")

if __name__ == "__main__":
    main()
