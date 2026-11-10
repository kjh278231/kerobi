#!/usr/bin/env python3
"""기존 대조군 검사 진입점. 목표 합격 검사는 별도의 acceptance 명령이다."""
import subprocess
import sys
from pathlib import Path

script = Path(__file__).with_name('mip_linear_child_accept.py')
sys.exit(subprocess.call([sys.executable, str(script), 'controls', *sys.argv[1:]]))
