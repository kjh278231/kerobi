Minimize
 obj: +0 xFR +0 yFR +0 xFN +0 xFU +0 yFU -1 xFD +0 yFD +0 zFD
Subject To
 rFR: -1 xFR +1 yFR <= -1
 rFU: +1 xFU +1 yFU <= 4
 rFD: -1 xFD +1 yFD <= -2
 tFD: +1e-08 xFD +1 zFD <= 1.00000003
Bounds
 -1e+30 <= xFR <= 1e+30
 0 <= yFR <= 10
 -1e+30 <= xFN <= 1e+30
 0 <= xFU <= 1e+30
 0 <= yFU <= 10
 0 <= xFD <= 1e+30
 0 <= yFD <= 10
 0 <= zFD <= 10
End
