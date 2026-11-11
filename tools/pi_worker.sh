#!/usr/bin/env bash
set -u

if (( $# < 1 || $# > 2 )) || [[ -z ${PI_WORKER_MODEL:-} ]]; then
    printf '%s\n' '사용법  PI_WORKER_MODEL=<모델 ID> tools/pi_worker.sh <프롬프트 파일> [이름]' >&2
    exit 2
fi

prompt=$1
if [[ ! -f $prompt || ! -r $prompt ]]; then
    printf '프롬프트 파일을 읽을 수 없습니다. %s\n' "$prompt" >&2
    exit 2
fi

name=${prompt##*/}
name=${2-${name%.md}}
if [[ -z $name || $name == */* ]]; then
    printf '%s\n' '이름은 비어 있지 않은 파일명이어야 합니다.' >&2
    exit 2
fi

script_dir=$(dirname -- "${BASH_SOURCE[0]}") || exit 1
mkdir -p .pi-run || exit 1
log=".pi-run/$name.jsonl"
{
    pi --mode json -p --no-session -a \
        --provider "${PI_WORKER_PROVIDER:-opengateway}" --model "$PI_WORKER_MODEL" \
        --tools read,bash,edit,write,grep,find,ls || :
} > "$log" 2> ".pi-run/$name.err" < "$prompt" || exit 1

check_args=("$log")
if [[ ${PI_WORKER_ALLOW_RE:-} == 1 ]]; then
    check_args+=(--allow-re)
fi
exec python3 "$script_dir/pi_log_check.py" "${check_args[@]}"
