classdef a_fleet
    % R2_FLEET  Value class describing a vehicle fleet with dependent totals.

    properties
        Name (1,1) string = "depot"
        Vehicles struct = struct('id', {}, 'km', {}, 'fuel', {})
        Capacity (1,1) double {mustBeInteger, mustBePositive} = 10
    end

    properties (Dependent)
        Count
        TotalKm
        MeanFuel
    end

    properties (Dependent, SetAccess = private)
        Full logical
    end

    methods
        function obj = a_fleet(name, capacity)
            if nargin >= 1
                obj.Name = name;
            end
            if nargin == 2
                obj.Capacity = capacity;
            end
        end

        function n = get.Count(obj)
            n = numel(obj.Vehicles);
        end

        function t = get.TotalKm(obj)
            if isempty(obj.Vehicles)
                t = 0;
            else
                t = sum([obj.Vehicles.km]);
            end
        end

        function m = get.MeanFuel(obj)
            m = mean([obj.Vehicles.fuel]);
        end

        function f = get.Full(obj)
            f = obj.Count >= obj.Capacity;
        end

        function obj = add(obj, id, km, fuel)
            if obj.Full
                error('r2:fleet:full', 'Fleet %s is full', obj.Name);
            end
            obj.Vehicles(end+1) = struct('id', id, 'km', km, 'fuel', fuel);
        end

        function obj = retire(obj, id)
            keep = [obj.Vehicles.id] ~= id;
            obj.Vehicles = obj.Vehicles(keep)
        end

        function disp(obj)
            fprintf('%s: %d vehicles, %.1f km\n', obj.Name, obj.Count, obj.TotalKm);
        end
    end
end
