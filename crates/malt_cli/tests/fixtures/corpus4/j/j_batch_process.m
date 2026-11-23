% j_batch_process.m
% Batch processing of the instrument files, written for the 2003 lab.
% Each raw file gives one .mat file, and a log line goes in batch.log.
% Run from the folder with the raw data.

clear all
close all
clc

datadir = pwd;
stamp = datestr(now, 'yyyymmdd_HHMM');
logname = ['batch_' stamp '.log'];
fid = fopen(logname, 'a');
if fid == -1
    error('could not open the log')
end
fprintf(fid, 'batch run started %s\n', datestr(now));

% pick the files, the pattern is typed at the prompt
pattern = input('file pattern (e.g. run*.dat): ', 's');
if isempty(pattern)
    pattern = '*.dat';
end
d = dir(pattern);
fprintf('%d files match %s\n', length(d), pattern)

nok = 0;
nbad = 0;
for i = 1:length(d)
    fname = d(i).name;
    fprintf('processing %s (%d of %d)\n', fname, i, length(d))
    try
        % load the raw file into a variable named after the file
        base = fname(1:end-4);
        vname = ['raw_' base];
        vname = strrep(vname, '-', '_');
        eval(['load(''' fname ''', ''-ascii'')'])
        eval(['tmp = ' base ';'])
        eval([vname ' = tmp;'])
        clear tmp
        eval(['clear ' base])

        % simple checks on the data
        X = eval(vname);
        if size(X, 2) < 2
            error('file has fewer than two columns')
        end
        if any(isnan(X(:)))
            fprintf(fid, '%s has NaN values\n', fname);
        end

        % a derived quantity, saved next to the raw data
        smooth = filter(ones(1, 5) / 5, 1, X(:, 2));
        peak = max(smooth);
        save([base '_proc.mat'], vname, 'smooth', 'peak')

        fprintf(fid, '%s ok, peak %g\n', fname, peak);
        nok = nok + 1;
    catch
        fprintf(fid, '%s FAILED: %s\n', fname, lasterr);
        nbad = nbad + 1;
    end
end

fprintf(fid, 'done: %d ok, %d failed\n', nok, nbad);
fclose(fid);

% gather all the peaks from the saved files for a summary table
peaks = [];
pd = dir('*_proc.mat');
for k = 1:length(pd)
    S = load(pd(k).name);
    peaks = [peaks; S.peak];
end
if ~isempty(peaks)
    fprintf('mean peak %g, std %g\n', mean(peaks), std(peaks))
end

% a dated summary file with the list of processed names
summary = ['summary_' stamp '.txt'];
sfid = fopen(summary, 'w');
for k = 1:length(d)
    fprintf(sfid, '%s\n', d(k).name);
end
fclose(sfid);

% the loaded variables are kept in a global workspace for the next step
global BATCH_PEAKS
BATCH_PEAKS = peaks;

% an old idiom for checking the output directory
if exist('processed', 'dir') ~= 7
    mkdir('processed')
end
movefile('*_proc.mat', 'processed')

% a manual pause, left in for the operator
% keyboard

% reload everything saved from the last session
load
who

% the time stamp of the newest file, for the report title
if ~isempty(pd)
    newest = pd(end).datenum;
    fprintf('newest result %s\n', datestr(newest, 0))
end
disp(['run finished in folder ' datadir])
