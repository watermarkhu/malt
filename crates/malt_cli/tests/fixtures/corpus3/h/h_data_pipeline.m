function [out, log] = h_data_pipeline(data, varargin)
%H_DATA_PIPELINE Run staged processing pipeline on measurement data
%   OUT = H_DATA_PIPELINE(DATA) runs the default pipeline on DATA.
%   OUT = H_DATA_PIPELINE(DATA, 'Name', Value, ...) overrides defaults.
%   OUT = H_DATA_PIPELINE(DATA, CFGSTRUCT) old style, still supported.
%
%   Stages: clean -> detrend -> filter -> segment -> summarise
%
%   See also: detrend, filtfilt

% TODO(jsmith): move config into a proper class once the GUI is retired
% NOTE: stage order matters, do not reorder without checking summarise

if nargin < 1 || isempty(data)
    error('h_data_pipeline:noData', 'No input data given.');
end

cfg = struct();
cfg.fs = 1000;
cfg.lowcut = 0.5;
cfg.highcut = 45;
cfg.segLen = 2;
cfg.verbose = true;
cfg.stages = {'clean', 'detrend', 'filter', 'segment', 'summarise'};

if numel(varargin) == 1 && isstruct(varargin{1})
    user = varargin{1};
    fn = fieldnames(user);
    for k = 1:numel(fn)
        cfg.(fn{k}) = user.(fn{k});
    end
else
    for k = 1:2:numel(varargin)
        cfg.(varargin{k}) = varargin{k+1};
    end
end

% defensive defaults, some of these were added after a bug report
if ~isfield(cfg, 'fs') || cfg.fs <= 0
    cfg.fs = 1000;
end
if ~isfield(cfg, 'lowcut')
    cfg.lowcut = 0.5;
end
if ~isfield(cfg, 'highcut') || cfg.highcut >= cfg.fs/2
    cfg.highcut = cfg.fs/2 - 1;
end
if ~isfield(cfg, 'segLen')
    cfg.segLen = 2;
end

log = {};
out = data;
nStages = numel(cfg.stages);
t0 = tic;
for s = 1:nStages
    stage = cfg.stages{s};
    try
        switch stage
            case 'clean'
                out = localClean(out);
            case 'detrend'
                out = localDetrend(out);
            case 'filter'
                out = localFilter(out, cfg.fs, cfg.lowcut, cfg.highcut);
            case 'segment'
                out = localSegment(out, cfg.fs, cfg.segLen);
            case 'summarise'
                out = localSummarise(out);
            otherwise
                warning('h_data_pipeline:unknownStage', 'Unknown stage %s', stage);
        end
        msg = sprintf('stage %-10s ok (%.3f s)', stage, toc(t0));
    catch ME
        msg = sprintf('stage %-10s FAILED: %s', stage, ME.message);
        if cfg.verbose
            fprintf(2, '%s\n', msg);
        end
        log{end+1} = msg; %#ok<AGROW>
        break
    end
    log{end+1} = msg; %#ok<AGROW>
    if cfg.verbose
        fprintf('[%d/%d] %s\n', s, nStages, msg);
    end
end

% old code path: segments came back as a cell, drop empty ones
if iscell(out)
    lens = cellfun(@(x) numel(x), out);
    out = out(lens > 0);
    names = cellfun(@(x) sprintf('seg%03d', x), num2cell(1:numel(out)), 'UniformOutput', false);
    fprintf('kept %d segments (%s ... %s)\n', numel(out), names{1}, names{end});
end

% copy-pasted from the old version, kept for the report script
if nargout < 2
    clear log
end
end

function x = localClean(x)
x(isnan(x)) = 0;
x(isinf(x)) = 0;
if size(x, 1) < size(x, 2)
    x = x.';
end
end

function x = localDetrend(x)
n = numel(x);
t = (1:n).';
p = polyfit(t, x(:), 1);
x = x(:) - polyval(p, t);
end

function y = localFilter(x, fs, lo, hi)
% butter needs normalised cutoffs, check they are sane first
wn = [lo hi] / (fs/2);
if any(wn <= 0) || any(wn >= 1)
    error('h_data_pipeline:band', 'Band [%g %g] outside Nyquist', lo, hi);
end
[b, a] = butter(4, wn, 'bandpass');
y = filtfilt(b, a, x);
end

function segs = localSegment(x, fs, segLen)
L = round(segLen * fs);
nSeg = floor(numel(x) / L);
segs = cell(1, nSeg);
for i = 1:nSeg
    idx = (i-1)*L + (1:L);
    segs{i} = x(idx);
end
end

function s = localSummarise(segs)
if ~iscell(segs)
    segs = {segs};
end
s = struct('mean', [], 'std', [], 'n', 0);
vals = cellfun(@mean, segs);
s.mean = mean(vals);
s.std = std(vals);
s.n = numel(segs);
end
