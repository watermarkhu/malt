classdef l_data_container < matlab.mixin.Copyable & matlab.mixin.SetGet
    %L_DATA_CONTAINER Handle-backed container of numeric samples.
    %   Copies are made with copy(), which calls copyElement() so that
    %   the sample vector is duplicated but the release callback is not.
    %   Setting Name or Tolerance while a copy is in progress aborts the
    %   assignment, which AbortSet makes cheap to rely on.

    properties (AbortSet)
        Name (1,1) string = "container"
        Tolerance (1,1) double {mustBeNonnegative, mustBeFinite} = 1e-6
    end

    properties (SetObservable)
        Values double = []
    end

    properties (Access = protected, NonCopyable)
        Cleanup = []
        Count (1,1) uint32 = 0
    end

    properties (GetAccess = protected, SetAccess = private)
        Created (1,1) datetime
    end

    methods
        function obj = l_data_container(name, values)
            %L_DATA_CONTAINER Construct a container with name and samples.
            arguments
                name (1,1) string = "container"
                values double = []
            end
            obj.Name = name;
            obj.Values = values;
            obj.Created = datetime('now');
            obj.Cleanup = onCleanup(@() obj.release());
        end

        function add(obj, x)
            %ADD Append one sample.
            obj.Values(end+1) = x;
            obj.Count = obj.Count + 1;
        end

        function tf = isWithin(obj, x)
            %ISWITHIN True when every sample lies within Tolerance of x.
            tf = all(abs(obj.Values - x) <= obj.Tolerance);
        end

        function c = clone(obj)
            c = obj.copy();
        end

        function n = sampleCount(obj)
            n = numel(obj.Values);
        end

        function delete(obj)
            %DELETE Release samples when the handle goes away.
            obj.release();
        end
    end

    methods (Access = protected)
        function cp = copyElement(obj)
            cp = copyElement@matlab.mixin.Copyable(obj);
            cp.Values = obj.Values;
            cp.Count = 0;
        end

        function release(obj)
            obj.Values = [];
        end
    end
end
