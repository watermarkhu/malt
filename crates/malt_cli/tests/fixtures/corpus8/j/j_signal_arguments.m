function [xs, ys] = j_signal_arguments(t, y, opts)
%R4_SIGNAL_ARGUMENTS Resample a signal onto a uniform grid, with options.

arguments
    t (:,1) double {mustBeFinite, mustBeIncreasing(t)}
    y (:,1) double
    opts.Rate (1,1) double {mustBePositive} = 10
    opts.Method (1,1) string = "linear"
    opts.Extrap (1,1) logical = false
end

if numel(t) ~= numel(y)
    error('r4:signal:length', 't and y need the same length.');
end

tEnd = t(end);
xs = (t(1):1/opts.Rate:tEnd)';
if opts.Extrap
    ys = interp1(t, y, xs, opts.Method, 'extrap');
else
    ys = interp1(t, y, xs, opts.Method);
end

ys(isnan(ys)) = 0;
ys = movmean(ys, 3);
spectrum = abs(fft(ys - mean(ys)));
[~, peakIdx] = max(spectrum(2:floor(end/2)));
peakHz = peakIdx * opts.Rate / numel(ys)
dt = diff(xs);
meanStep = mean(dt)
nanCount = sum(isnan(y))
switch opts.Method
    case {"linear", "nearest"}
        note = "simple";
    case "spline"
        note = "smooth";
    otherwise
        note = "other";
end
end

function mustBeIncreasing(v)
    if any(diff(v) <= 0)
        error('r4:signal:order', 'time must increase');
    end
end
