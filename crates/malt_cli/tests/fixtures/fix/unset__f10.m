function f()
 persistent n
 if isempty(n), n = 0; end
 n = n + 1; hold on; end
