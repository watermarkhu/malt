classdef c_sensor_bus < handle
    % R4_SENSOR_BUS  Publish/subscribe bus for sensor readings with listeners.

    events
        Reading
        Overflow
    end

    properties (SetAccess = private)
        Count (1,1) uint32 = 0
        Limit (1,1) double = 100
    end

    properties (Access = private)
        Buffer double = []
        Subscribers cell = cell(1, 0)
    end

    methods
        function obj = c_sensor_bus(limit)
            if nargin == 1
                obj.Limit = limit;
            end
        end

        function id = subscribe(obj, fn)
            obj.Subscribers{end+1} = fn;
            id = numel(obj.Subscribers);
        end

        function publish(obj, value)
            obj.Count = obj.Count + 1;
            if numel(obj.Buffer) >= obj.Limit
                notify(obj, 'Overflow');
                obj.Buffer(1) = [];
            end
            obj.Buffer(end+1) = value;
            notify(obj, 'Reading', sensorEvent_(value));
            for k = 1:numel(obj.Subscribers)
                obj.Subscribers{k}(value);
            end
        end

        function m = meanValue(obj)
            if isempty(obj.Buffer)
                m = NaN;
            else
                m = mean(obj.Buffer);
            end
        end

        function delete(obj)
            obj.Subscribers = {};
        end
    end
end

function ed = sensorEvent_(value)
    ed = struct('value', value, 'when', now);
end
