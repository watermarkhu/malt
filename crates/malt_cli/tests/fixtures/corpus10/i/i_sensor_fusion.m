classdef i_sensor_fusion < handle
    % R6_SENSOR_FUSION combines readings from several sensors with a Kalman-like gain.

    properties (Constant)
        DefaultVariance = 0.25
    end

    properties
        Estimate (1,1) double = 0
        Variance (1,1) double {mustBePositive} = 1
        History double = []
    end

    properties (Dependent)
        Confidence
    end

    methods
        function obj = i_sensor_fusion(initial)
            arguments
                initial (1,1) double = 0
            end
            obj.Estimate = initial;
        end

        function c = get.Confidence(obj)
            c = 1 / (1 + obj.Variance);
        end

        function update(obj, z, r)
            arguments
                obj
                z (1,1) double
                r (1,1) double {mustBePositive} = i_sensor_fusion.DefaultVariance
            end
            k = obj.Variance / (obj.Variance + r);
            obj.Estimate = obj.Estimate + k * (z - obj.Estimate);
            obj.Variance = (1 - k) * obj.Variance;
            obj.History(end+1) = obj.Estimate;
        end

        function out = fuse(obj, readings, variances)
            for i = 1:numel(readings)
                obj.update(readings(i), variances(i));
            end
            out = obj.Estimate;
        end

        function reset(obj)
            obj.Estimate = 0;
            obj.Variance = 1;
            obj.History = [];
        end
    end

    methods (Static)
        function est = batch(readings)
            fuser = i_sensor_fusion;
            est = zeros(size(readings));
            for k = 1:numel(readings)
                fuser.update(readings(k));
                est(k) = fuser.Estimate;
            end
        end
    end
end
