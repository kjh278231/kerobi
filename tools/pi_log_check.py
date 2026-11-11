#!/usr/bin/env python3
"""pi JSONL 로그에서 작업자 결과와 금지 호출을 확인한다."""

import io
import json
import sys
from pathlib import Path


SOLVER_MARKERS = ("gurobi", "gurun.sh", "grbgetkey")
REFERENCE_MARKERS = ("re/decomp", "re/notes", "../re/", "dc.py")
USAGE = "사용법  python3 tools/pi_log_check.py <log.jsonl> [--allow-re] 또는 --self-test"


def reject_constant(value):
    raise ValueError(value)


def check_log(lines, allow_re, output):
    last_assistant = None
    tokens = calls = tool_errors = retries = 0
    failed = False

    for line_number, line in enumerate(lines, 1):
        try:
            event = json.loads(line.decode("utf-8"), parse_constant=reject_constant)
            event_type = event["type"]
            if not isinstance(event_type, str):
                raise ValueError

            if event_type == "message_end":
                message = event["message"]
                usage = message.get("usage", {}).get("totalTokens", 0)
                if type(usage) is not int or usage < 0:
                    raise ValueError
                if message["role"] == "assistant":
                    final_text = "\n".join(
                        block["text"] for block in message["content"] if block["type"] == "text"
                    )
                    stop_reason = message["stopReason"]
                    if not isinstance(stop_reason, str):
                        raise ValueError
                    last_assistant = final_text, stop_reason
                tokens += usage
            elif event_type == "tool_execution_start":
                args = event["args"]
                tool = json.dumps(event["toolName"])
                encoded_args = json.dumps(args)
                command = args.get("command", "")
                if not isinstance(command, str):
                    raise ValueError
                calls += 1
                if any(marker in command for marker in SOLVER_MARKERS) or (
                    not allow_re and any(marker in encoded_args for marker in REFERENCE_MARKERS)
                ):
                    print(f"{line_number}행 금지 도구 호출 {tool} {encoded_args}", file=output)
                    failed = True
            elif event_type == "tool_execution_end":
                tool_errors += event["isError"] is True
            elif event_type == "auto_retry_start":
                retries += 1
        except (AttributeError, KeyError, TypeError, ValueError):
            print(f"{line_number}행 JSONL 파싱 실패", file=output)
            failed = True

    if last_assistant is None:
        print("assistant message_end가 없습니다.", file=output)
        failed = True
    else:
        final_text, stop_reason = last_assistant
        print("마지막 assistant 메시지", file=output)
        print(final_text, file=output)
        if stop_reason in ("error", "aborted"):
            print(f"마지막 assistant 메시지의 종료 사유는 {stop_reason}입니다.", file=output)
            failed = True

    print(f"토큰 사용량 합계 {tokens}", file=output)
    print(f"도구 호출 {calls}건", file=output)
    print(f"도구 오류 {tool_errors}건", file=output)
    print(f"자동 재시도 {retries}건", file=output)
    return int(failed)


def self_test():
    def message(stop_reason="stop", text="완료\u2028다음 줄", tokens=7):
        return {
            "type": "message_end",
            "message": {
                "role": "assistant",
                "content": [{"type": "text", "text": text}],
                "stopReason": stop_reason,
                "usage": {"totalTokens": tokens},
            },
        }

    def tool(name, **args):
        return {"type": "tool_execution_start", "toolName": name, "args": args}

    def run(*records, allow_re=False):
        data = b"\n".join(
            record if isinstance(record, bytes) else json.dumps(record, ensure_ascii=False).encode()
            for record in records
        )
        output = io.StringIO()
        code = check_log(io.BytesIO(data), allow_re, output)
        return code, output.getvalue()

    code, output = run(
        message("error", "이전 응답", 2),
        {"type": "auto_retry_start"},
        tool("bash", command="pwd"),
        {"type": "tool_execution_end", "isError": True},
        message(),
    )
    assert code == 0 and "완료\u2028다음 줄" in output and "이전 응답" not in output
    assert "토큰 사용량 합계 9\n도구 호출 1건\n도구 오류 1건\n자동 재시도 1건" in output
    assert run({"type": "agent_settled", "aborted": False})[0] == 1
    for reason in ("error", "aborted"):
        assert run(message(), message(reason))[0] == 1
    for invalid in (b"{", b"[]", b"\xff", b"", b'{"type":"event","data":NaN}'):
        assert run(message(), invalid, message())[0] == 1
    for command in ("gurobi_cl model.lp", "tools/gurun.sh model.lp", "grbgetkey"):
        assert run(tool("bash", command=command), message(), allow_re=True)[0] == 1
    references = [tool("read", path=path) for path in ("re/decomp/file.c", "re/notes/file", "../re/file")]
    references.append(tool("bash", command="python3 tools/dc.py 함수"))
    code, output = run(*references, message())
    assert code == 1 and output.count("금지 도구 호출") == len(references)
    assert run(*references, message(), allow_re=True)[0] == 0
    print("자가 시험 통과")


def main(args):
    if args == ["--self-test"]:
        self_test()
        return 0
    if args in (["--help"], ["-h"]):
        print(USAGE)
        return 0
    allow_re = "--allow-re" in args
    if allow_re:
        args.remove("--allow-re")
    if len(args) != 1 or args[0].startswith("--"):
        print(USAGE, file=sys.stderr)
        return 2

    try:
        with Path(args[0]).open("rb") as log:
            return check_log(log, allow_re, sys.stdout)
    except OSError:
        print(f"로그 파일을 읽을 수 없습니다. {args[0]}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
