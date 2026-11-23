function result = f_network_client(endpoint, payload, maxRetries)
%F_NETWORK_CLIENT Post a JSON payload to a REST endpoint with retries.
%   RESULT = F_NETWORK_CLIENT(ENDPOINT, PAYLOAD) sends the struct PAYLOAD
%   as JSON and returns a struct with fields ok, status and body.
%
%   RESULT = F_NETWORK_CLIENT(ENDPOINT, PAYLOAD, MAXRETRIES) changes the
%   number of attempts (default 3).

if nargin < 3
    maxRetries = 3;
end

opts = weboptions('MediaType', 'application/json', ...
    'Timeout', 15, 'RequestMethod', 'post', ...
    'HeaderFields', {'Accept', 'application/json'});

result = struct('ok', false, 'status', NaN, 'body', []);
body = jsonencode(payload);

for attempt = 1:maxRetries
    try
        raw = webwrite(endpoint, body, opts);
        result.status = 200;

        % The service sometimes returns plain text instead of JSON
        if ischar(raw) || isstring(raw)
            result.body = jsondecode(char(raw));
        else
            result.body = raw;
        end

        % The API reports logical errors inside a 200 response
        if isstruct(result.body) && isfield(result.body, 'error')
            fprintf('Server reported: %s\n', result.body.error);
            result.ok = false;
        else
            result.ok = true;
        end
        break
    catch ME
        fprintf('Attempt %d/%d failed: %s\n', attempt, maxRetries, ME.message);

        % Pull the HTTP status out of the error if the server sent one
        if contains(ME.identifier, 'HTTP')
            result.status = str2double(regexp(ME.message, '\d{3}', 'match', 'once'));
        end

        if attempt < maxRetries
            pause(2^attempt);   % exponential backoff
        end
    end
end

if ~result.ok
    warning('f_network_client:failed', ...
        'Request to %s failed after %d attempts', endpoint, maxRetries);
end
end
