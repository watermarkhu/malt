classdef h_instrument_handle < handle
    %H_INSTRUMENT_HANDLE Driver wrapper for a VISA-attached instrument
    %   Handles connection, polling via timer and status events.
    %
    %   d = h_instrument_handle('TCPIP0::192.168.1.20::INSTR', 'Poll', 0.25);
    %   d.connect();
    %   d.start();

    properties (SetAccess = private)
        Address char
        Connected logical = false
        LastReading double = NaN
    end

    properties (SetObservable)
        Status (1,:) char = 'idle'
    end

    properties (Access = protected)
        Port
        Timer
        PollPeriod = 0.5
        RetryCount = 3;
    end

    events
        ReadingReady
        StatusChanged
    end

    methods
        function obj = h_instrument_handle(address, varargin)
            if nargin < 1
                address = 'GPIB0::12::INSTR';
            end
            obj.Address = address;

            p = inputParser;
            p.FunctionName = 'h_instrument_handle';
            addParameter(p, 'Poll', 0.5);
            addParameter(p, 'Retries', 3);
            parse(p, varargin{:});
            obj.PollPeriod = p.Results.Poll;
            obj.RetryCount = p.Results.Retries;

            obj.Timer = timer('ExecutionMode', 'fixedSpacing', ...
                'Period', obj.PollPeriod, ...
                'TimerFcn', @obj.onTick, ...
                'ErrorFcn', @(src, ev) obj.onError(ev), ...
                'StopFcn', @(~,~) disp('poll stopped'));
        end

        function set.Status(obj, value)
            if ~ischar(value) && ~isstring(value)
                error('h_instrument_handle:badStatus', 'Status must be text');
            end
            old = obj.Status;
            obj.Status = char(value);
            if ~strcmp(old, obj.Status)
                notify(obj, 'StatusChanged');
            end
        end

        function connect(obj)
            if obj.Connected
                warning('h_instrument_handle:alreadyConnected', 'Already connected to %s', obj.Address);
                return
            end
            attempt = 0;
            while attempt < obj.RetryCount
                attempt = attempt + 1;
                try
                    obj.Port = visadev(obj.Address);
                    obj.Connected = true;
                    break
                catch err
                    fprintf('attempt %d failed: %s\n', attempt, err.message);
                    pause(0.2 * attempt);
                end
            end
            if ~obj.Connected
                error('h_instrument_handle:connectFailed', ...
                    'Could not reach %s after %d tries', obj.Address, obj.RetryCount);
            end
            obj.Status = 'connected';
        end

        function disconnect(obj)
            obj.stop();
            obj.Port = [];
            obj.Connected = false;
            obj.Status = 'disconnected';
        end

        function start(obj)
            if ~obj.Connected
                obj.connect();
            end
            if strcmp(obj.Timer.Running, 'off')
                start(obj.Timer);
            end
        end

        function stop(obj)
            if ~isempty(obj.Timer) && isvalid(obj.Timer)
                if strcmp(obj.Timer.Running, 'on')
                    stop(obj.Timer);
                end
            end
        end

        function onTick(obj, ~, ~)
            try
                raw = obj.query('MEAS?');
                obj.LastReading = str2double(raw);
                notify(obj, 'ReadingReady');
            catch err
                obj.onError(err);
            end
        end

        function onError(obj, ev)
            obj.Status = ['error: ' ev.message];
            warning('h_instrument_handle:tick', '%s', ev.message);
        end

        function resp = query(obj, cmd)
            if ~obj.Connected
                error('h_instrument_handle:notConnected', 'Not connected');
            end
            writeline(obj.Port, cmd);
            resp = strtrim(readline(obj.Port));
        end

        function lh = addReadingListener(obj, fcn)
            lh = addlistener(obj, 'ReadingReady', fcn);
        end

        function delete(obj)
            % called when the last reference goes away, be quiet about it
            if ~isempty(obj.Timer) && isvalid(obj.Timer)
                stop(obj.Timer);
                delete(obj.Timer);
            end
        end
    end
end
