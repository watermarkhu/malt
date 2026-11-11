function [price, delta, mcPrice, mcStdErr] = d_option_pricing(S, K, T, r, sigma, optType, nSims)
%D_OPTION_PRICING Price a European option by Black-Scholes and Monte Carlo.
%   price = D_OPTION_PRICING(S, K, T, r, sigma) returns the Black-Scholes
%   price of a European call with spot S, strike K, maturity T (years),
%   risk-free rate r and volatility sigma.
%
%   D_OPTION_PRICING(..., optType) with optType 'call' or 'put'.
%   D_OPTION_PRICING(..., optType, nSims) also runs a Monte Carlo estimate
%   with nSims paths (default 1e5) and returns its mean and standard error.

if nargin < 6 || isempty(optType)
    optType = 'call';
end
if nargin < 7
    nSims = 1e5;
end

% Black-Scholes d1 and d2
d1 = (log(S ./ K) + (r + 0.5 * sigma.^2) .* T) ./ (sigma .* sqrt(T));
d2 = d1 - sigma .* sqrt(T);

switch lower(optType)
    case 'call'
        price = S .* normcdf(d1) - K .* exp(-r .* T) .* normcdf(d2);
        delta = normcdf(d1);
    case 'put'
        price = K .* exp(-r .* T) .* normcdf(-d2) - S .* normcdf(-d1);
        delta = normcdf(d1) - 1;
    otherwise
        error('d_option_pricing:badType', 'Unknown option type "%s".', optType);
end

% Monte Carlo under the risk-neutral measure
if nargout > 2
    z = randn(nSims, 1);
    ST = S * exp((r - 0.5 * sigma^2) * T + sigma * sqrt(T) * z);
    payoff = payoffVector(ST, K, optType);
    discounted = exp(-r * T) * payoff;
    mcPrice = mean(discounted);
    mcStdErr = std(discounted) / sqrt(nSims);
end

end

function p = payoffVector(ST, K, optType)
% Vectorised payoff for calls and puts
if strcmpi(optType, 'call')
    p = max(ST - K, 0);
else
    p = max(K - ST, 0);
end
end
