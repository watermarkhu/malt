function results = f_simulink_helper(modelName, gainValues)
%F_SIMULINK_HELPER Sweep a gain parameter in a Simulink model.
%   RESULTS = F_SIMULINK_HELPER(MODELNAME) runs the default sweep.
%   RESULTS = F_SIMULINK_HELPER(MODELNAME, GAINVALUES) runs one simulation
%   per gain value and returns the logged signal of each run.

if nargin < 2
    gainValues = [0.5 1.0 2.0];
end

results = struct('gain', {}, 'signal', {}, 'ok', {});

try
    load_system(modelName);

    % Find the first gain block directly under the top level
    blocks = find_system(modelName, 'SearchDepth', 1, ...
        'BlockType', 'Gain');
    if isempty(blocks)
        error('f_simulink_helper:noGain', ...
            'No Gain block found in %s', modelName);
    end
    gainBlock = blocks{1};
    fprintf('Using block %s\n', gainBlock);

    stopTime = get_param(modelName, 'StopTime');
    fprintf('Model stop time: %s s\n', stopTime);

    for k = 1:numel(gainValues)
        % Build a SimulationInput so each run is independent
        simIn = Simulink.SimulationInput(modelName);
        simIn = simIn.setModelParameter('StopTime', '5');
        simIn = simIn.setBlockParameter(gainBlock, 'Gain', ...
            num2str(gainValues(k)));
        simIn = simIn.setModelParameter('SignalLogging', 'on');

        out = sim(simIn);

        row = struct('gain', gainValues(k), 'signal', [], 'ok', false);
        if isprop(out, 'logsout')
            sig = out.logsout.get('y');
            row.signal = sig.Values.Data;
            row.ok = true;
        end
        results(end+1) = row; %#ok<AGROW>
    end

    set_param(modelName, 'Dirty', 'off');
    close_system(modelName, 0);
catch ME
    warning('f_simulink_helper:failed', '%s', ME.message);
    results = [];
    % Make sure the model does not stay open after a failure
    if bdIsLoaded(modelName)
        bdclose(modelName);
    end
end
end
