function data = f_serial_comm(portName, nSamples, timeoutSec)
%F_SERIAL_COMM Read a block of ASCII samples from a serial instrument.
%   DATA = F_SERIAL_COMM(PORTNAME, NSAMPLES) opens PORTNAME at 115200 baud,
%   requests NSAMPLES readings and returns them as a column vector.
%
%   DATA = F_SERIAL_COMM(PORTNAME, NSAMPLES, TIMEOUTSEC) also sets the
%   maximum time to wait for the instrument (default 10 s).
%
%   Example:
%     d = f_serial_comm("COM3", 100, 5);

if nargin < 3 || isempty(timeoutSec)
    timeoutSec = 10;
end

% Preallocate the output so the loop does not grow it
data = nan(nSamples, 1);
stamp = datetime('now', 'Format', 'yyyy-MM-dd HH:mm:ss');
fprintf('[%s] Opening %s\n', char(stamp), portName);

s = [];
try
    s = serialport(portName, 115200);
    configureTerminator(s, "LF");
    s.Timeout = 2;
    flush(s);

    % Ask the instrument to start streaming
    writeline(s, "START");

    tStart = tic;
    k = 0;
    while k < nSamples && toc(tStart) < timeoutSec
        try
            line = readline(s);
        catch readErr
            % Timeouts are expected when the device is slow
            warning('f_serial_comm:readTimeout', ...
                'Read failed after %d samples: %s', k, readErr.message);
            break
        end

        value = str2double(strtrim(line));
        if isnan(value)
            continue
        end
        k = k + 1;
        data(k) = value;
    end

    writeline(s, "STOP");
    data = data(1:k);
    fprintf('Collected %d of %d samples in %.2f s\n', k, nSamples, toc(tStart));
catch err
    fprintf(2, 'Serial acquisition failed: %s\n', err.message);
    rethrow(err)
end

% Release the port explicitly
clear s
end
