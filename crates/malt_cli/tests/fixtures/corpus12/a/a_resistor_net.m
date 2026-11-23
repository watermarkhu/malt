function [req, currents] = resistor_net(R, V)
%RESISTOR_NET Solves a resistor network by nodal analysis.
%   R is an n-by-n conductance-free resistance matrix (0 means no edge).

n = size(R, 1);
G = zeros(n);
for i = 1:n
    for j = 1:n
        if i ~= j && R(i, j) > 0
            G(i, j) = -1 / R(i, j);
            G(i, i) = G(i, i) + 1 / R(i, j);
        end
    end
end
G(1, :) = 0;
G(1, 1) = 1;
rhs = zeros(n, 1);
rhs(1) = V;
if rcond(G) < 1e-12
    warning('net:singular', 'network is badly conditioned');
end
volt = G \ rhs;
req = V / sum(volt(2:end) ./ volt(2:end));
currents = zeros(n);
for i = 1:n
    for j = 1:n
        if R(i, j) > 0
            currents(i, j) = (volt(i) - volt(j)) / R(i, j);
        end
    end
end
display(req)
total = trace(currents)
end
