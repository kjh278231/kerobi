# 작업별 진행 파일

스프린트 branch는 이 폴더에 `<작업번호>.md` 한 파일만 만든다(예: `M2-06g.md`). `docs/11_progress.md`·`12_roadmap_diagram.md`·`13_work_units.md`·`14_sprint_plan.md`와 `docs/handoff/M*.md` 맨 위는 `tools/progress_index.py`가 이 파일들로 다시 만든다. 병합은 `tools/merge_task.py`가 한다.

```
date: 2026-11-11T09:30
area: M2
## 11
11_progress 맨 위에 들어갈 문단 하나.
## 12
12_roadmap_diagram 맨 위에 들어갈 문단 하나.
## 13
| M2-06 | 이름 | 상태 | 링크 |
## 14
| M2-06g | 상태 | 이름 | ... |
## handoff
다음 작업 명령과 인수인계(영역당 최신 3개가 인수인계 파일 위에 나온다).
```

- 구역은 모두 선택이다. `date`와 `area`는 필수이며 `date`가 새로울수록 위에 놓인다.
- 13·14 행은 첫 칸이 같은 기존 행을 대체하고, 없으면 그 영역 표의 마지막 행 뒤에 들어간다.
- 점검: `python3 tools/progress_index.py --check`(밀려 있으면 종료 1), 자체 시험 `--self-test`.
