function result = k_spacey_style( inputData , options )
% K_SPACEY_STYLE  Summarise a numeric input with aligned assignments.
%
%   result = k_spacey_style( inputData , options )
%   options may have fields scale, offset and label.

    if nargin < 2 || isempty( options )
        options = struct( );
    end

    defaultScale   =  1.0;
    defaultOffset  =  0.0;
    defaultLabel   =  'untitled';
    maxIterations  =  50;

    scale   = getfield_or( options , 'scale'  , defaultScale  );
    offset  = getfield_or( options , 'offset' , defaultOffset );
    label   = getfield_or( options , 'label'  , defaultLabel  );

    values  = inputData( : ) * scale + offset;
    count   = numel( values );
    total   = sum( values );
    average = total / max( count , 1 );

    tally = zeros( 1 , maxIterations );
    for  k = 1 : min( count , maxIterations )
        tally( k ) = values( k ) ^ 2;
    end

    if  average  >  0
        sign_text = 'positive';
    elseif  average  <  0
        sign_text = 'negative';
    else
        sign_text = 'zero';
    end

    % build the result struct, one field per line
    result.label    = label;
    result.count    = count;
    result.total    = total;
    result.average  = average;
    result.sign     = sign_text;
    result.squares  = tally( 1 : min( count , maxIterations ) );

    result.summary  = sprintf( '%s: n=%d mean=%.3f' , label , count , average );
    fprintf( '%s\n' , result.summary );

    % quick sanity check on the spread
    spread = max( values ) - min( values );
    if  spread  ==  0
        warning( 'k:flat' , 'all values identical' );
    end
    result.spread   = spread;

    % second pass, normalised
    normed = values ./ max( abs( values ) , eps );
    result.normed   = normed( : ).';
    result.checksum = sum( normed , 'all' );

end

function value = getfield_or( s , name , fallback )
    if  isfield( s , name )
        value = s.(name);
    else
        value = fallback;
    end
end

function out = clamp_to( v , lo , hi )
    out = min( max( v , lo ) , hi );
end
