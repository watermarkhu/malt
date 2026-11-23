function f()
 a = 1;
 b = a(end) + numel(a) + a.x;
 s = struct(); s(2).k = 1; disp(s);
 c = cell(1); c{end+1} = 2;
 fprintf("%d", b); x = @() a;
 [q, w] = deal(1); disp(q)
 for k = 1:3, end
 disp(k)
 end
