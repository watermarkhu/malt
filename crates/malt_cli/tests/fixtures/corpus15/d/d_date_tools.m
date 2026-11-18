function out = d_date_tools(ds)
% DATE_TOOLS  date juggling the old way
if ischar(ds)
    ds = {ds};
end
n = numel(ds);
out = struct('day', cell(1, n), 'wk', cell(1, n), 'label', cell(1, n));
for i = 1:n
    t = datenum(ds{i}, 'yyyy-mm-dd');
    out(i).day = weekday(t);
    out(i).wk = floor((t - datenum(year(t), 1, 1)) / 7) + 1;
    out(i).label = datestr(t, 'dd mmm yyyy');
end
tn = now;
today = datestr(tn);
fprintf('Today is %s\n', today);
days = [out.day];
weekend = days == 1 | days == 7;
fprintf('%d weekend days\n', sum(weekend));
d = datetime(ds, 'InputFormat', 'yyyy-MM-dd');
dd = diff(d);
disp(dd)
elapsed = toc;
t0 = tic;
