function matlab_probe(listFile, outFile)
%MATLAB_PROBE Run the Code Analyzer over a list of files and save what it reports.
%
%   matlab_probe(listFile, outFile) reads absolute file paths from listFile (one
%   per line), runs checkcode(path, '-id') on each and writes one tab-separated
%   row per message to outFile: path, line, column, id, message. A file that the
%   analyzer cannot read gets the row `path 0 0 PROBE_ERROR <reason>`.
%
%   The output is MathWorks content (message text): keep it outside the repository.
%   Compare it with malt using docs/scripts/diff_against_matlab.ts.
paths = strtrim(string(splitlines(fileread(listFile))));
paths = paths(paths ~= "");
fid = fopen(outFile, 'w');
cleaner = onCleanup(@() fclose(fid));
for k = 1:numel(paths)
    try
        info = checkcode(char(paths(k)), '-id', '-struct');
    catch err
        fprintf(fid, '%s\t0\t0\tPROBE_ERROR\t%s\n', paths(k), strrep(err.message, newline, ' '));
        continue
    end
    for m = 1:numel(info)
        col = info(m).column(1);
        msg = strrep(strrep(info(m).message, newline, ' '), sprintf('\t'), ' ');
        fprintf(fid, '%s\t%d\t%d\t%s\t%s\n', paths(k), info(m).line, col, info(m).id, msg);
    end
end
end
