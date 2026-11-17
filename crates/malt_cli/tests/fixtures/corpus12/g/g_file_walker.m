function counts = file_walker(root, pattern)
%FILE_WALKER Counts lines matching a pattern in every .txt file under root.

if ~isfolder(root)
    error('walk:notFolder', '%s is not a folder', root);
end

counts = containers.Map();
pending = {root};
while ~isempty(pending)
    current = pending{end};
    pending(end) = [];
    entries = dir(current);
    for k = 1:numel(entries)
        name = entries(k).name;
        if any(strcmp(name, {'.', '..'}))
            continue
        end
        full = fullfile(current, name);
        if entries(k).isdir
            pending{end+1} = full;
        elseif endsWith(name, '.txt')
            fid = fopen(full, 'r');
            hits = 0;
            line = fgetl(fid);
            while ischar(line)
                if ~isempty(regexp(line, pattern, 'once'))
                    hits = hits + 1;
                end
                line = fgetl(fid);
            end
            fclose(fid);
            counts(full) = hits;
        end
    end
end

total = sum(cell2mat(values(counts)));
if total == 0
    fprintf('no matches under %s\n', root);
end
listing = keys(counts);
clear full
end
