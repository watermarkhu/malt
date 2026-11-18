classdef a_temperature_sensor < handle
    % TEMPERATURE_SENSOR  Simple sensor model with validated properties.
    properties
        Name (1,1) string = "unnamed"
        Offset (1,1) double {mustBeFinite} = 0
    end
    properties (SetAccess = private)
        Readings (:,1) double = []
    end
    properties (Dependent)
        Mean
        Count
    end
    properties (Constant)
        MaxReadings = 1000
    end
    events
        NewReading
    end
    methods
        function obj = a_temperature_sensor(name, varargin)
            if nargin > 0
                obj.Name = name;
            end
            for k = 1:2:numel(varargin)
                obj.(varargin{k}) = varargin{k+1};
            end
        end
        function m = get.Mean(obj)
            if isempty(obj.Readings)
                m = NaN;
            else
                m = mean(obj.Readings) + obj.Offset;
            end
        end
        function n = get.Count(obj)
            n = length(obj.Readings);
        end
        function add(obj, value)
            if numel(obj.Readings) >= obj.MaxReadings
                obj.Readings(1) = [];
            end
            obj.Readings(end+1) = value;
            notify(obj, 'NewReading');
        end
        function reset(obj)
            obj.Readings = [];
        end
        function disp(obj)
            fprintf('Sensor %s: %d readings, mean %.2f\n', obj.Name, obj.Count, obj.Mean);
        end
        function tf = eq(a, b)
            tf = strcmp(a.Name, b.Name) && a.Offset == b.Offset;
        end
    end
    methods (Static)
        function obj = fromData(name, data)
            obj = a_temperature_sensor(name);
            for k = 1:numel(data)
                obj.add(data(k));
            end
        end
    end
end
