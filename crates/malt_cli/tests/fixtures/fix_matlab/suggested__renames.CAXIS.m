function out = renames(n, xs, ys, cmap, path1)
%RENAMES Direct-replacement function renames, calls and bare names.
spmd
    piece = labindex:numlabs:n;
    local = gop(@plus, sum(xs(piece)));
    fprintf('worker %d of %d holds %d\n', labindex, numlabs, numel(piece));
end
clims = [min(ys) max(ys)];
clim(clims)
flag = isdir(path1);
flag2 = isfolder(path1);
out = [sum([local{:}]), flag, flag2, numel(xs), cmap];
end
