%[text] # Sinusoidal Signals
%[text] Examples of sinusoidal signals in MATLAB.
%[text:tableOfContents]{"heading":"Contents"}
%[text] - sine waves
%[text] - cosine waves \
x = linspace(0,8*pi);
%%
%[text] ## Sine Wave
plot(x,sin(x))
title('Sine Wave')
grid on
%%
%[text] ## Cosine Wave
plot(x,cos(x))
%[output:6f1a2b3c]{"layout":"inline"}
%   data: {"layout":"inline"}
%---
%%
%[text] ## Summary
%[text] The sine and cosine functions are $ \\pi/2 $ radians out of phase.

%[appendix]{"version":"1.0"}
%---
%[metadata:view]
%   data: {"layout":"inline"}
%---
