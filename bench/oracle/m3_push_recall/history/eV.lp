Minimize
 obj: +0 xw +0 yw +0 xVR +0 yVR +0 xVN +0 xVU +0 yVU +0 xVD +0 yVD +0 wVD +0 xVDd +0 yVDd +0 wVDd +0 xVS +0 yVS +0 wVS +0 xVRg +0 yVRg
Subject To
 cw: +1 xw +1 yw <= 1.7
 rVR: -1 xVR +1 yVR <= -1
 rVU: +1 xVU +1 yVU <= 4
 bVD: +1 wVD -2e-08 xVD <= 0.94999993999999999
 rVD: -1 xVD +1 yVD <= -2
 bVDd: +1 wVDd -2e-08 xVDd <= 0.94999998999999991
 rVDd: -1 xVDd +1 yVDd <= 0.5
 bVS: +1 wVS +2e-08 xVS <= 0.95000003999999993
 rVS: +1 xVS +1 yVS <= 3
 rVRg: -0.00050000000000000001 xVRg +1 yVRg <= -0.00089999999999999998
Bounds
 0 <= xw <= 1
 0 <= yw <= 1
 -1e+30 <= xVR <= 1e+30
 0 <= yVR <= 10
 -1e+30 <= xVN <= 1e+30
 0 <= xVU <= 1e+30
 0 <= yVU <= 10
 0 <= xVD <= 1e+30
 0 <= yVD <= 10
 0 <= wVD <= 1
 0 <= xVDd <= 1e+30
 0 <= yVDd <= 10
 0 <= wVDd <= 1
 -1e+30 <= xVS <= 1e+30
 0 <= yVS <= 10
 0 <= wVS <= 1
 -1e+30 <= xVRg <= 1e+30
 0 <= yVRg <= 10
End
