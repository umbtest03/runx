#!/usr/bin/env python3
import datetime
import json
import sys


def main() -> int:
    invocation = json.load(sys.stdin)

    inputs = invocation.get("inputs", {})
    response = {
        "schema": "runx.external_adapter.response.v1",
        "protocol_version": "runx.external_adapter.v1",
        "invocation_id": invocation["invocation_id"],
        "adapter_id": invocation["adapter_id"],
        "status": "completed",
        "stdout": json.dumps({"message": inputs.get("message")}),
        "stderr": "",
        "exit_code": 0,
        "output": {
            "adapter_language": "python",
            "message": inputs.get("message"),
            "count": inputs.get("count"),
        },
        "observed_at": datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z"),
    }
    sys.stdout.write(json.dumps(response, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
