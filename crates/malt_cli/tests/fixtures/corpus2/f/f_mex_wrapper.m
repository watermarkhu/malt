function y = f_mex_wrapper(x, opts)
%F_MEX_WRAPPER Dispatch a moving-mean smoother to MEX or MATLAB code.
%   Y = F_MEX_WRAPPER(X) uses the compiled mymex function when it is on the
%   path and falls back to a pure MATLAB implementation otherwise.
%
%   Y = F_MEX_WRAPPER(X, OPTS) accepts OPTS.window (default 5).

if nargin < 2 || ~isstruct(opts)
    opts = struct();
end
if ~isfield(opts, 'window')
    opts.window = 5;
end

% Only look for the MEX file when running in MATLAB, not in codegen
if coder.target('MATLAB')
    hasMex = exist('mymex', 'file') == 3;
else
    hasMex = false;
end

if hasMex
    y = mymex(x, opts.window);
    return
end

fprintf('mymex not found, using MATLAB fallback on %s\n', computer('arch'));
y = movmean(x, opts.window);

% Check for a C compiler, which the user needs to build mymex
if isunix
    [status, out] = system('gcc --version');
    if status ~= 0
        fprintf('No C compiler detected: %s\n', strtok(out, newline));
    end
end

% Tell the user how to build the MEX file on their platform
if ismac
    hint = 'Run make -f Makefile.mac';
elseif ispc
    hint = 'Run build_mymex.bat from the Developer Prompt';
elseif isunix
    hint = 'Run make -f Makefile.linux';
else
    hint = 'Unknown platform';
end
fprintf('Hint: %s\n', hint);
end
