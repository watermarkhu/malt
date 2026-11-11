function [xHat, P, nu] = d_kalman_filter(z, F, H, Q, R, reset)
%D_KALMAN_FILTER Linear Kalman filter step with persistent state.
%   [xHat, P, nu] = D_KALMAN_FILTER(z, F, H, Q, R) advances the filter by
%   one measurement z and returns the posterior state estimate xHat, the
%   posterior covariance P and the innovation nu.
%
%   [...] = D_KALMAN_FILTER(z, F, H, Q, R, true) resets the internal state
%   before processing z. The state is initialised to zero with covariance
%   1e3*eye(n) on the first call.
%
%   Inputs
%     z  - measurement vector (m x 1)
%     F  - state transition matrix (n x n)
%     H  - measurement matrix (m x n)
%     Q  - process noise covariance (n x n)
%     R  - measurement noise covariance (m x m)
%
%   See also KALMANFILTER.

persistent x P initialised

if nargin < 6
    reset = false;
end

n = size(F, 1);
m = size(H, 1);

% Input checks
if size(F, 2) ~= n
    error('d_kalman_filter:size', 'F must be square, got %dx%d.', size(F, 1), size(F, 2));
end
if size(H, 2) ~= n
    error('d_kalman_filter:size', 'H must have %d columns.', n);
end
if ~isequal(size(Q), [n n]) || ~isequal(size(R), [m m])
    error('d_kalman_filter:size', 'Q and R must be %dx%d and %dx%d.', n, n, m, m);
end
if numel(z) ~= m
    error('d_kalman_filter:size', 'Measurement must have %d elements.', m);
end

% Initialise state on first use or when asked to reset
if reset || isempty(initialised)
    x = zeros(n, 1);
    P = 1e3 * eye(n);
    initialised = true;
end

% Predict
xPred = F * x;
PPred = F * P * F' + Q;

% Update
nu = z(:) - H * xPred;
S = H * PPred * H' + R;
K = PPred * H' / S;

x = xPred + K * nu;
P = (eye(n) - K * H) * PPred;

% Symmetrise to limit round-off drift
P = 0.5 * (P + P');

xHat = x;
end
