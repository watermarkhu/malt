---
icon: lucide/lightbulb
---

# Suggested Improvements

**Default severity:** Info
**Auto-fix:** Yes
**Category:** Suggested Improvements
**Can be disabled:** Yes
**Enabled by default:** Yes
**Maturity:** Stable

## What this rule does

A data-driven rule module that handles 243 MATLAB Code Analyzer checks
about functions and patterns that are not recommended, pairing each with a
modern replacement suggestion. Instead of implementing one struct per
check, a single `SuggestedImprovementsEngine` loads
`data/suggested_improvements.toml` at compile time, parses it into a
function-name lookup table, and emits diagnostics with the specific check
ID from the matched entry (e.g. `CSVRD`, not the meta ID
`SUGGESTED_IMPROVEMENTS`).

## Check IDs

### Suggested Improvements (243 checks)

| Check ID | Severity | Maturity | Description |
| -------- | -------- | -------- | ----------- |
| <a id="importdyn"></a>`IMPORTDYN` | warning | stable | Using function syntax to call 'import' is not recommended. With appropriate code changes, use command syntax instead. |
| <a id="lerr"></a>`LERR` | warning | stable | LASTERR and LASTERROR are not recommended. Use an identifier on the CATCH block instead. |
| <a id="evlc"></a>`EVLC` | warning | stable | Using 'evalc' with two arguments is not recommended. Use try/catch statements instead to make code more clear and efficient. |
| <a id="rand"></a>`RAND` | warning | stable | RAND or RANDN with the 'seed', 'state', or 'twister' inputs is not recommended. Use RNG instead. |
| <a id="rand"></a>`RAND` | warning | stable | RAND or RANDN with the 'seed', 'state', or 'twister' inputs is not recommended. Use RNG instead. |
| <a id="hough"></a>`HOUGH` | warning | stable | HOUGH(BW,'ThetaResolution',VAL) is not recommended. Use HOUGH(BW,'Theta',-90:VAL:(90-VAL) ) instead. |
| <a id="thour"></a>`THOUR` | warning | stable | 'hour' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' as input instead. |
| <a id="tmnth"></a>`TMNTH` | warning | stable | 'month' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' as input instead. |
| <a id="tmnut"></a>`TMNUT` | warning | stable | 'minute' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' as input instead. |
| <a id="tnday"></a>`TNDAY` | warning | stable | 'day' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' as input instead. |
| <a id="tscnd"></a>`TSCND` | warning | stable | 'second' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' as input instead. |
| <a id="tqurt"></a>`TQURT` | warning | stable | 'quarter' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' as input instead. |
| <a id="tyear"></a>`TYEAR` | warning | stable | 'year' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' as input instead. |
| <a id="tdtvec"></a>`TDTVEC` | warning | stable | 'datevec' with serial date number or text inputs is not recommended. With appropriate code changes, use 'datetime' instead. |
| <a id="tnow1"></a>`TNOW1` | warning | stable | 'now' is not recommended. With appropriate code changes, use 'datetime(\ |
| <a id="tnow2"></a>`TNOW2` | warning | stable | 'datetime(now, 'ConvertFrom', 'datenum')' is not recommended. Use 'datetime(\ |
| <a id="ttday1"></a>`TTDAY1` | warning | stable | 'today' is not recommended. With appropriate code changes, use 'datetime(\ |
| <a id="ttday2"></a>`TTDAY2` | warning | stable | 'datetime(today, 'ConvertFrom', 'datenum')' is not recommended. Use 'datetime(\ |
| <a id="match2"></a>`MATCH2` | warning | stable | STRMATCH is not recommended. Use STRNCMP or VALIDATESTRING instead. |
| <a id="match2"></a>`MATCH2` | warning | stable | STRMATCH is not recommended. Use STRNCMP or VALIDATESTRING instead. |
| <a id="imgdt"></a>`IMGDT` | warning | stable | Using 'DataAugmentation' in function 'imageInputLayer' is not recommended. Use function 'augmentedImageDatastore' instead. |
| <a id="match3"></a>`MATCH3` | warning | stable | STRMATCH is not recommended. Use STRCMP instead. |
| <a id="mtfa1"></a>`MTFA1` | warning | stable | MAKETFORM('AFFINE',A) is not recommended. Use AFFINE2D or AFFINE3D instead. |
| <a id="mtfa2"></a>`MTFA2` | warning | stable | MAKETFORM('AFFINE',U,X) is not recommended. Use FITGEOTRANS instead. |
| <a id="mtfp1"></a>`MTFP1` | warning | stable | MAKETFORM('PROJECTIVE',A) is not recommended. Use PROJECTIVE2D instead. |
| <a id="mtfp2"></a>`MTFP2` | warning | stable | MAKETFORM('PROJECTIVE',U,X) is not recommended. Use FITGEOTRANS instead. |
| <a id="mtfb"></a>`MTFB` | warning | stable | MAKETFORM('BOX',...) is not recommended. Use IMREF2D or IMREF3D instead. |
| <a id="oops"></a>`OOPS` | warning | stable | Defining a class using 'function' syntax is not recommended. With appropriate code changes, use 'classdef' syntax instead. |
| <a id="pmtmconf"></a>`PMTMCONF` | warning | stable | When using PMTM with three output arguments, the 'ConfidenceLevel' input argument is recommended. |
| <a id="nchki"></a>`NCHKI` | warning | stable | NARGCHK is not recommended. Use NARGINCHK instead. |
| <a id="nchko"></a>`NCHKO` | warning | stable | Using NARGCHK with NARGOUT is not recommended. Use NARGOUTCHK instead. |
| <a id="nchkn"></a>`NCHKN` | warning | stable | NARGCHK is not recommended. Use NARGINCHK without ERROR instead. |
| <a id="nchkm"></a>`NCHKM` | warning | stable | NARGCHK is not recommended. Use NARGOUTCHK without ERROR instead. |
| <a id="isclstr"></a>`ISCLSTR` | warning | stable | To support string in addition to cellstr, include a call to 'isstring'. |
| <a id="emtag"></a>`EMTAG` | warning | stable | The compilation directive (or pragma) 'eml' is not recommended. Use 'codegen' instead. |
| <a id="emxtr"></a>`EMXTR` | warning | stable | The 'eml' namespace is not recommended. Use 'codegen' instead. |
| <a id="nvrepla"></a>`NVREPLA` | warning | stable | 'addParamValue' is not recommended. Use 'addParameter' instead. |
| <a id="nvreplm"></a>`NVREPLM` | warning | stable | 'MidPctRef' is not recommended. Use 'MidPercentReferenceLevel' instead. |
| <a id="nvreplp"></a>`NVREPLP` | warning | stable | 'PctRefLevels' is not recommended. Use 'PercentReferenceLevels' instead. |
| <a id="vidread"></a>`VIDREAD` | warning | stable | 'NumberOfFrames' is not recommended. Use 'NumFrames' instead. |
| <a id="crnr"></a>`CRNR` | warning | stable | CORNER is not recommended. Use detectHarrisFeatures or detectMinEigenFeatures in Computer Vision Toolbox instead. |
| <a id="crnrm"></a>`CRNRM` | warning | stable | CORNERMETRIC is not recommended. Use detectHarrisFeatures or detectMinEigenFeatures and the cornerPoints class in Computer Vision Toolbox instead. |
| <a id="mdflt1"></a>`MDFLT1` | warning | stable | BLKSZ is required for backward compatibility and is ignored. Use [] instead. |
| <a id="intrpp"></a>`INTRPP` | warning | stable | 'pp' is not recommended. Use the griddedInterpolant class instead. |
| <a id="displayprog"></a>`DISPLAYPROG` | warning | stable | Programmatic use of DISPLAY is not recommended. Use DISP or FPRINTF instead. |
| <a id="hgstgt"></a>`HGSTGT` | warning | stable | hgsetget is not recommended. Use matlab.mixin.SetGet or matlab.mixin.SetGetExactNames instead. |
| <a id="legacymd"></a>`LEGACYMD` | warning | stable | Setting LegacyMode to true is not recommended. Set LegacyMode to false instead. |
| <a id="legacytrd"></a>`LEGACYTRD` | warning | stable | 'DetectorMethod' is applicable only when LegacyMode is set to true. However, setting LegacyMode to true is not recommended. |
| <a id="legacytrl"></a>`LEGACYTRL` | warning | stable | 'LoopMethod' is applicable only when LegacyMode is set to true. However, setting LegacyMode to true is not recommended. |
| <a id="legacytru"></a>`LEGACYTRU` | warning | stable | 'UpdatePeriod' is applicable only when LegacyMode is set to true. However, setting LegacyMode to true is not recommended. |
| <a id="legacytrs"></a>`LEGACYTRS` | warning | stable | 'StepSize' is applicable only when LegacyMode is set to true. However, setting LegacyMode to true is not recommended. |
| <a id="legacytrg"></a>`LEGACYTRG` | warning | stable | 'GainOutputPort' is applicable only when LegacyMode is set to true. However, setting LegacyMode to true is not recommended. |
| <a id="ezplt"></a>`EZPLT` | warning | stable | EZPLOT is not recommended. Use FPLOT or FIMPLICIT instead. |
| <a id="ezgrph3"></a>`EZGRPH3` | warning | stable | EZGRAPH3 is not recommended. Use FCONTOUR, FMESH, FPLOT, FPLOT3 or FSURF instead. |
| <a id="ezcntrf"></a>`EZCNTRF` | warning | stable | EZCONTOURF is not recommended. Use FCONTOUR instead, and set the 'Fill' value to 'on'. |
| <a id="ezmshc"></a>`EZMSHC` | warning | stable | EZMESHC is not recommended. Use FMESH instead, and set the 'ShowContours' value to 'on'. |
| <a id="ezsrfc"></a>`EZSRFC` | warning | stable | EZSURFC is not recommended. Use FSURF instead, and set the 'ShowContours' value to 'on'. |
| <a id="fisadr"></a>`FISADR` | warning | stable | 'addrule' is not recommended. Use 'addRule' instead. |
| <a id="strquot"></a>`STRQUOT` | warning | stable | string('...') is not recommended. Use \ |
| <a id="strclqt"></a>`STRCLQT` | warning | stable | 'string({'str1', 'str2'})' is not recommended. Use '[\ |
| <a id="sim"></a>`SIM` | warning | stable | 'sim' in parfor loop is not recommended. Replace the parfor loop with 'parsim'. |
| <a id="numch"></a>`NUMCH` | warning | stable | 'NumberOfChannels' is not recommended. Use 'NumChannels' instead. |
| <a id="gtred"></a>`GTRED` | warning | stable | 'geotiffread' is not recommended, except when reading a GeoTIFF file from a URL. With appropriate code changes, use 'readgeoraster' instead. |
| <a id="getfsp"></a>`GETFSP` | warning | stable | Using get for retrieving values of line spacing is not recommended. With appropriate code changes, use 'settings' object instead. |
| <a id="setfmt"></a>`SETFMT` | warning | stable | Using set for assigning values of numeric display format is not recommended. With appropriate code changes, use 'settings' object instead. |
| <a id="setfsp"></a>`SETFSP` | warning | stable | Using set for assigning values of line spacing is not recommended. With appropriate code changes, use 'settings' object instead. |
| <a id="getfmt"></a>`GETFMT` | warning | stable | Using get for retrieving values of numeric display format is not recommended. With appropriate code changes, use 'settings' object instead. |
| <a id="ev2in"></a>`EV2IN` | warning | stable | Using 'eval' with two arguments is not recommended. Use try/catch statements instead to make code more clear and efficient. |
| <a id="ev3in"></a>`EV3IN` | warning | stable | Using 'evalin' with three arguments is not recommended. Use try/catch statements instead to make code more clear and efficient. |
| <a id="prtog"></a>`PRTOG` | warning | stable | '-opengl' is not recommended. Use '-image' instead, which is a direct replacement. |
| <a id="prtpt"></a>`PRTPT` | warning | stable | '-painters' is not recommended. Use '-vector' instead, which is a direct replacement. |
| <a id="formatnoi"></a>`FORMATNOI` | warning | stable | 'format' with no input or output arguments is not recommended. Use 'format(\ |
| <a id="xfrwsb"></a>`XFRWSB` | warning | stable | The 'TransferBaseWorkspaceVariables' option is not recommended for 'batchsim'. With appropriate code changes, consider using project startup scripts or the 'SetupFcn' option instead. |
| <a id="xfrwsp"></a>`XFRWSP` | warning | stable | The 'TransferBaseWorkspaceVariables' option is not recommended for 'parsim'. With appropriate code changes, consider using project startup scripts or the 'SetupFcn' option instead. |
| <a id="oldsim"></a>`OLDSIM` | warning | stable | This syntax of the 'sim' command which returns multiple arguments is not recommended. With appropriate code changes, turn on 'ReturnWorkspaceOutputs' and return simulation results using the single-output format instead. |
| <a id="insthwi"></a>`INSTHWI` | warning | stable | 'instrhwinfo('ivi')' is not recommended. With appropriate code changes, use 'ividriverlist' or 'ividevlist' instead. |
| <a id="inwvx"></a>`INWVX` | warning | stable | 'instrhwinfo('vxipnp')' is not recommended. With appropriate code changes, use 'ividriverlist' or 'ividevlist' instead. |
| <a id="vermatlab"></a>`VERMATLAB` | warning | stable | ver('matlab') is not recommended. With appropriate code changes, use 'matlabRelease' instead. |
| <a id="verlessmatlab"></a>`VERLESSMATLAB` | warning | stable | verLessThan('matlab', ...) is not recommended. With appropriate code changes, use 'isMATLABReleaseOlderThan' instead. |
| <a id="risklmm"></a>`RISKLMM` | warning | stable | 'Model' property of 'risk.credit.pd.LifetimePDModel' class is not recommended. Use 'UnderlyingModel' property of 'risk.credit.pd.LifetimePDModel' class instead. |
| <a id="risklma"></a>`RISKLMA` | warning | stable | 'modelAccuracy' method of 'risk.credit.pd.LifetimePDModel' class is not recommended. Use 'modelCalibration' method of 'risk.credit.pd.LifetimePDModel' class instead. |
| <a id="risklmap"></a>`RISKLMAP` | warning | stable | 'modelAccuracyPlot' method of 'risk.credit.pd.LifetimePDModel' class is not recommended. Use 'modelCalibrationPlot' method of 'risk.credit.pd.LifetimePDModel' class instead. |
| <a id="risklgdma"></a>`RISKLGDMA` | warning | stable | 'modelAccuracy' method of 'risk.credit.lgd.LGDModel' class is not recommended. Use 'modelCalibration' method of 'risk.credit.lgd.LGDModel' class instead. |
| <a id="risklgdmap"></a>`RISKLGDMAP` | warning | stable | 'modelAccuracyPlot' method of 'risk.credit.lgd.LGDModel' class is not recommended. Use 'modelCalibrationPlot' method of 'risk.credit.lgd.LGDModel' class instead. |
| <a id="riskeadma"></a>`RISKEADMA` | warning | stable | 'modelAccuracy' method of 'risk.credit.ead.EADModel' class is not recommended. Use 'modelCalibration' method of 'risk.credit.ead.EADModel' class instead. |
| <a id="riskeadmap"></a>`RISKEADMAP` | warning | stable | 'modelAccuracyPlot' method of 'risk.credit.ead.EADModel' class is not recommended. Use 'modelCalibrationPlot' method of 'risk.credit.ead.EADModel' class instead. |
| <a id="holdall"></a>`HOLDALL` | warning | stable | 'hold('all')' is not recommended. Use 'hold('on')' instead, which is a direct replacement. |
| <a id="masso"></a>`MASSO` | warning | stable | 'MasterSolverOptions' is not recommended. Use 'MainSolverOptions' instead, which is a direct replacement. |
| <a id="imsso"></a>`IMSSO` | warning | stable | 'IntMasterSolverOptions' is not recommended. Use 'IntMainSolverOptions' instead, which is a direct replacement. |
| <a id="cdfepoch2date"></a>`CDFEPOCH2DATE` | warning | stable | 'ConvertEpochToDatenum' is not recommended. With appropriate code changes, use the 'DatetimeType' parameter of 'cdfread' instead. |
| <a id="featgpid"></a>`FEATGPID` | warning | stable | 'feature('getpid')' is unsupported and not recommended. With appropriate code changes, use the function 'matlabProcessID' instead. |
| <a id="dtrirep"></a>`DTRIREP` | warning | stable | 'TriRep' is not recommended. With appropriate code changes, use 'triangulation' instead. |
| <a id="ddeltri"></a>`DDELTRI` | warning | stable | 'DelaunayTri' is not recommended. With appropriate code changes, use 'delaunayTriangulation' instead. |
| <a id="dtriint"></a>`DTRIINT` | warning | stable | 'TriScatteredInterp' is not recommended. With appropriate code changes, use 'scatteredInterpolant' instead. |
| <a id="dapplut"></a>`DAPPLUT` | warning | stable | 'applylut' is not recommended. With appropriate code changes, use 'bwlookup' instead. |
| <a id="dblkprc"></a>`DBLKPRC` | warning | stable | 'blkproc' is not recommended. With appropriate code changes, use 'blockproc' instead. |
| <a id="caxis"></a>`CAXIS` | warning | stable | 'caxis' is not recommended. Use 'clim' instead, which is a direct replacement. |
| <a id="cdfepoch"></a>`CDFEPOCH` | warning | stable | 'cdfepoch' is not recommended. With appropriate code changes, use 'cdflib' low-level functions instead. |
| <a id="todatenum"></a>`TODATENUM` | warning | stable | 'todatenum' is not recommended. With appropriate code changes, use the 'DatetimeType' parameter of 'cdfread' instead. |
| <a id="commpamm"></a>`COMMPAMM` | warning | stable | 'comm.PAMModulator' is not recommended. With appropriate code changes, use 'pammod' instead. |
| <a id="commpamd"></a>`COMMPAMD` | warning | stable | 'comm.PAMDemodulator' is not recommended. With appropriate code changes, use 'pamdemod' instead. |
| <a id="commsrc"></a>`COMMSRC` | warning | stable | 'commsrc.pn' is not recommended. With appropriate code changes, use 'comm.PNSequence' instead. |
| <a id="dcptf"></a>`DCPTF` | warning | stable | 'cp2tform' is not recommended. With appropriate code changes, use 'fitgeotrans' instead. |
| <a id="datnm"></a>`DATNM` | warning | stable | 'datenum' is not recommended. With appropriate code changes, use 'datetime' instead. |
| <a id="datst"></a>`DATST` | warning | stable | 'datestr' is not recommended. With appropriate code changes, use 'datetime' instead. |
| <a id="detim"></a>`DETIM` | warning | stable | 'etime' is not recommended. With appropriate code changes, use 'datetime' and the minus operator instead. |
| <a id="datod"></a>`DATOD` | warning | stable | 'addtodate' is not recommended. With appropriate code changes, use 'datetime', 'duration', and the plus operator instead. |
| <a id="clock"></a>`CLOCK` | warning | stable | 'clock' is not recommended. With appropriate code changes, use 'datetime(\ |
| <a id="date"></a>`DATE` | warning | stable | 'date' is not recommended. With appropriate code changes, use 'datetime(\ |
| <a id="datic"></a>`DATIC` | warning | stable | 'datetick' is not recommended. With appropriate code changes, use datetime and duration arrays directly in charts. Modify display using 'xtickformat', 'ytickformat', or 'ztickformat'. |
| <a id="wknum"></a>`WKNUM` | warning | stable | 'weeknum' is not recommended. With appropriate code changes, use 'week' with a 'datetime' input instead. |
| <a id="emdate"></a>`EMDATE` | warning | stable | 'eomdate' is not recommended. With appropriate code changes, use 'dateshift' with a 'datetime' input instead. |
| <a id="xmdate"></a>`XMDATE` | warning | stable | 'x2mdate' is not recommended. With appropriate code changes, use 'datetime(..., \ |
| <a id="mxdate"></a>`MXDATE` | warning | stable | 'm2xdate' is not recommended. With appropriate code changes, use 'exceltime' with a 'datetime' input instead. |
| <a id="mnths"></a>`MNTHS` | warning | stable | 'months' is not recommended. With appropriate code changes, use 'between' with a 'datetime' input instead. |
| <a id="dgtord"></a>`DGTORD` | warning | stable | 'degtorad' is not recommended. Use 'deg2rad' instead, which is a direct replacement. |
| <a id="rdtodg"></a>`RDTODG` | warning | stable | 'radtodeg' is not recommended. Use 'rad2deg' instead, which is a direct replacement. |
| <a id="ezcontr"></a>`EZCONTR` | warning | stable | 'ezcontour' is not recommended. With appropriate code changes, use 'fcontour' instead. |
| <a id="ezmesh"></a>`EZMESH` | warning | stable | 'ezmesh' is not recommended. With appropriate code changes, use 'fmesh' instead. |
| <a id="ezplt3"></a>`EZPLT3` | warning | stable | 'ezplot3' is not recommended. With appropriate code changes, use 'fplot3' instead. |
| <a id="ezsurf"></a>`EZSURF` | warning | stable | 'ezsurf' is not recommended. With appropriate code changes, use 'fsurf' instead. |
| <a id="ezpolar"></a>`EZPOLAR` | warning | stable | 'ezpolar' is not recommended. With appropriate code changes, use 'fpolarplot' instead. |
| <a id="compass"></a>`COMPASS` | warning | stable | 'compass' is not recommended. With appropriate code changes, use 'compassplot' instead. |
| <a id="dflipdim"></a>`DFLIPDIM` | warning | stable | 'flipdim' is not recommended. With appropriate code changes, use 'flip' instead. |
| <a id="dstrmt"></a>`DSTRMT` | warning | stable | 'str2mat' is not recommended. With appropriate code changes, use 'char' instead. |
| <a id="dststr"></a>`DSTSTR` | warning | stable | 'setstr' is not recommended. With appropriate code changes, use 'char' instead. |
| <a id="dstrvct"></a>`DSTRVCT` | warning | stable | 'strvcat' is not recommended. With appropriate code changes, use 'char' instead. |
| <a id="disstr"></a>`DISSTR` | warning | stable | 'isstr' is not recommended. With appropriate code changes, use 'ischar' instead. |
| <a id="dftsmtx"></a>`DFTSMTX` | warning | stable | 'fts2mtx' is not recommended. With appropriate code changes, use 'fts2mat' instead. |
| <a id="fiswrt"></a>`FISWRT` | warning | stable | 'writefis' is not recommended. Use 'writeFIS' instead, which is a direct replacement. |
| <a id="fisadm"></a>`FISADM` | warning | stable | 'addmf' is not recommended. With appropriate code changes, use 'addMF' instead. |
| <a id="hist"></a>`HIST` | warning | stable | 'hist' is not recommended. With appropriate code changes, use 'histogram' instead. |
| <a id="histc"></a>`HISTC` | warning | stable | 'histc' is not recommended. With appropriate code changes, use 'histcounts' instead. |
| <a id="rose"></a>`ROSE` | warning | stable | 'rose' is not recommended. With appropriate code changes, use 'polarhistogram' instead. |
| <a id="h5cls"></a>`H5CLS` | warning | stable | 'H5.close' is not recommended and no longer has any effect. There is no simple replacement for this. |
| <a id="h5opn"></a>`H5OPN` | warning | stable | 'H5.open' is not recommended and no longer has any effect. There is no simple replacement for this. |
| <a id="hdfi"></a>`HDFI` | warning | stable | 'hdf5info' is not recommended. With appropriate code changes, use 'h5info' instead. |
| <a id="hdfw"></a>`HDFW` | warning | stable | 'hdf5write' is not recommended. With appropriate code changes, use 'h5write' instead. |
| <a id="hdfr"></a>`HDFR` | warning | stable | 'hdf5read' is not recommended. With appropriate code changes, use 'h5read' instead. |
| <a id="im2bw"></a>`IM2BW` | warning | stable | 'im2bw' is not recommended. With appropriate code changes, use 'imbinarize' instead. |
| <a id="imfreeh"></a>`IMFREEH` | warning | stable | 'imfreehand' is not recommended. With appropriate code changes, use 'drawfreehand' instead. |
| <a id="imrect"></a>`IMRECT` | warning | stable | 'imrect' is not recommended. With appropriate code changes, use 'drawrectangle' instead. |
| <a id="imline"></a>`IMLINE` | warning | stable | 'imline' is not recommended. With appropriate code changes, use 'drawline' instead. |
| <a id="impnt"></a>`IMPNT` | warning | stable | 'impoint' is not recommended. With appropriate code changes, use 'drawpoint' instead. |
| <a id="impoly"></a>`IMPOLY` | warning | stable | 'impoly' is not recommended. With appropriate code changes, use 'drawpolygon' or 'drawpolyline' instead. |
| <a id="imellps"></a>`IMELLPS` | warning | stable | 'imellipse' is not recommended. With appropriate code changes, use 'drawellipse' or 'drawcircle' instead. |
| <a id="dimtrns"></a>`DIMTRNS` | warning | stable | 'imtransform' is not recommended. With appropriate code changes, use 'imwarp' instead. |
| <a id="fileattrib"></a>`FILEATTRIB` | warning | stable | 'fileattrib' is not recommended. With appropriate code changes, use 'filePermissions' instead. |
| <a id="csvrd"></a>`CSVRD` | warning | stable | 'csvread' is not recommended. With appropriate code changes, use 'readtable' or 'readmatrix' instead. |
| <a id="dlmrd"></a>`DLMRD` | warning | stable | 'dlmread' is not recommended. With appropriate code changes, use 'readtable' or 'readmatrix' instead. |
| <a id="csvwt"></a>`CSVWT` | warning | stable | 'csvwrite' is not recommended. With appropriate code changes, use 'writematrix' instead. |
| <a id="dlmwt"></a>`DLMWT` | warning | stable | 'dlmwrite' is not recommended. With appropriate code changes, use 'writematrix' instead. |
| <a id="xlsrd"></a>`XLSRD` | warning | stable | 'xlsread' is not recommended. With appropriate code changes, use 'readtable', 'readmatrix' or 'readcell' instead. |
| <a id="xlswt"></a>`XLSWT` | warning | stable | 'xlswrite' is not recommended. With appropriate code changes, use 'writematrix' or 'writecell' instead. |
| <a id="isdir"></a>`ISDIR` | warning | stable | 'isdir' is not recommended. Use 'isfolder' instead, which is a direct replacement. |
| <a id="diseqn"></a>`DISEQN` | warning | stable | 'isequalwithequalnans' is not recommended. With appropriate code changes, use 'isequaln' instead. |
| <a id="dgcat"></a>`DGCAT` | warning | stable | 'gcat' is not recommended. Use 'spmdCat' instead, which is a direct replacement. |
| <a id="dgop"></a>`DGOP` | warning | stable | 'gop' is not recommended. Use 'spmdReduce' instead, which is a direct replacement. |
| <a id="dgplus"></a>`DGPLUS` | warning | stable | 'gplus' is not recommended. Use 'spmdPlus' instead, which is a direct replacement. |
| <a id="dlabbarrier"></a>`DLABBARRIER` | warning | stable | 'labBarrier' is not recommended. Use 'spmdBarrier' instead, which is a direct replacement. |
| <a id="dlabbroadcast"></a>`DLABBROADCAST` | warning | stable | 'labBroadcast' is not recommended. Use 'spmdBroadcast' instead, which is a direct replacement. |
| <a id="dlabindex"></a>`DLABINDEX` | warning | stable | 'labindex' is not recommended. Use 'spmdIndex' instead, which is a direct replacement. |
| <a id="dlabprobe"></a>`DLABPROBE` | warning | stable | 'labProbe' is not recommended. Use 'spmdProbe' instead, which is a direct replacement. |
| <a id="dlabreceive"></a>`DLABRECEIVE` | warning | stable | 'labReceive' is not recommended. Use 'spmdReceive' instead, which is a direct replacement. |
| <a id="dlabsend"></a>`DLABSEND` | warning | stable | 'labSend' is not recommended. Use 'spmdSend' instead, which is a direct replacement. |
| <a id="dlabsendreceive"></a>`DLABSENDRECEIVE` | warning | stable | 'labSendReceive' is not recommended. Use 'spmdSendReceive' instead, which is a direct replacement. |
| <a id="dnumlabs"></a>`DNUMLABS` | warning | stable | 'numlabs' is not recommended. Use 'spmdSize' instead, which is a direct replacement. |
| <a id="agred"></a>`AGRED` | warning | stable | 'arcgridread' is not recommended. With appropriate code changes, use 'readgeoraster' instead. |
| <a id="mdfobj"></a>`MDFOBJ` | warning | stable | 'mdf' is not recommended. With appropriate code changes, use 'mdfInfo', 'mdfChannelGroupInfo' or 'mdfChannelInfo' instead. |
| <a id="ddbmex"></a>`DDBMEX` | warning | stable | 'mexdebug' is not recommended. With appropriate code changes, use 'dbmex' instead. |
| <a id="mlnt"></a>`MLNT` | warning | stable | 'mlint' is not recommended. Use 'checkcode' instead, which is a direct replacement. |
| <a id="mnrfit"></a>`MNRFIT` | warning | stable | 'mnrfit' is not recommended. With appropriate code changes, use 'fitmnr' instead. |
| <a id="mnrval"></a>`MNRVAL` | warning | stable | 'mnrval' is not recommended. With appropriate code changes, use 'MultinomialRegression.predict' instead. |
| <a id="mustinrange"></a>`MUSTINRANGE` | warning | stable | 'mustBeInRange' is not recommended. With appropriate code changes, use 'mustBeBetween' instead. |
| <a id="depnoe"></a>`DEPNOE` | warning | stable | 'numberofelements' is not recommended. With appropriate code changes, use 'numel' instead. |
| <a id="gaopt"></a>`GAOPT` | warning | stable | 'gaoptimset' is not recommended. With appropriate code changes, use 'optimoptions' instead. |
| <a id="psopt"></a>`PSOPT` | warning | stable | 'psoptimset' is not recommended. With appropriate code changes, use 'optimoptions' instead. |
| <a id="saopt"></a>`SAOPT` | warning | stable | 'saoptimset' is not recommended. With appropriate code changes, use 'optimoptions' instead. |
| <a id="plotyy"></a>`PLOTYY` | warning | stable | 'plotyy' is not recommended. With appropriate code changes, use 'yyaxis' instead. |
| <a id="polar"></a>`POLAR` | warning | stable | 'polar' (MATLAB) is not recommended. Use 'polarplot' instead. |
| <a id="plbl"></a>`PLBL` | warning | stable | 'polybool' is not recommended. With appropriate code changes, use 'polyshape' instead. |
| <a id="pyver"></a>`PYVER` | warning | stable | 'pyversion' is not recommended. With appropriate code changes, use 'pyenv' instead. |
| <a id="dquad"></a>`DQUAD` | warning | stable | 'quad' is not recommended. With appropriate code changes, use 'integral' instead. |
| <a id="dquadl"></a>`DQUADL` | warning | stable | 'quadl' is not recommended. With appropriate code changes, use 'integral' instead. |
| <a id="dquadv"></a>`DQUADV` | warning | stable | 'quadv' is not recommended. With appropriate code changes, use 'integral' instead. |
| <a id="ddblqd"></a>`DDBLQD` | warning | stable | 'dblquad' is not recommended. With appropriate code changes, use 'integral2' instead. |
| <a id="dtriqd"></a>`DTRIQD` | warning | stable | 'triplequad' is not recommended. With appropriate code changes, use 'integral3' instead. |
| <a id="dfirrcos"></a>`DFIRRCOS` | warning | stable | 'firrcos' is not recommended. With appropriate code changes, use 'rcosdesign' instead. |
| <a id="dfirgauss"></a>`DFIRGAUSS` | warning | stable | 'firgauss' is not recommended. With appropriate code changes, use 'gaussdesign' instead. |
| <a id="dgaussfir"></a>`DGAUSSFIR` | warning | stable | 'gaussfir' is not recommended. With appropriate code changes, use 'gaussdesign' instead. |
| <a id="roifill"></a>`ROIFILL` | warning | stable | 'roifill' is not recommended. With appropriate code changes, use 'regionfill' instead. |
| <a id="depbart"></a>`DEPBART` | warning | stable | 'sigwin.barthannwin' is not recommended. With appropriate code changes, use 'barthannwin' instead. |
| <a id="deplett"></a>`DEPLETT` | warning | stable | 'sigwin.bartlett' is not recommended. With appropriate code changes, use 'bartlett' instead. |
| <a id="dblkmn"></a>`DBLKMN` | warning | stable | 'sigwin.blackman' is not recommended. With appropriate code changes, use 'blackman' instead. |
| <a id="dbhrrs"></a>`DBHRRS` | warning | stable | 'sigwin.blackmanharris' is not recommended. With appropriate code changes, use 'blackmanharris' instead. |
| <a id="dbhmnwn"></a>`DBHMNWN` | warning | stable | 'sigwin.bohmanwin' is not recommended. With appropriate code changes, use 'bohmanwin' instead. |
| <a id="dchbwn"></a>`DCHBWN` | warning | stable | 'sigwin.chebwin' is not recommended. With appropriate code changes, use 'chebwin' instead. |
| <a id="dflttpwn"></a>`DFLTTPWN` | warning | stable | 'sigwin.flattopwin' is not recommended. With appropriate code changes, use 'flattopwin' instead. |
| <a id="dgswin"></a>`DGSWIN` | warning | stable | 'sigwin.gausswin' is not recommended. With appropriate code changes, use 'gausswin' instead. |
| <a id="dhmmng"></a>`DHMMNG` | warning | stable | 'sigwin.hamming' is not recommended. With appropriate code changes, use 'hamming' instead. |
| <a id="dhann"></a>`DHANN` | warning | stable | 'sigwin.hann' is not recommended. With appropriate code changes, use 'hann' instead. |
| <a id="dkser"></a>`DKSER` | warning | stable | 'sigwin.kaiser' is not recommended. With appropriate code changes, use 'kaiser' instead. |
| <a id="dnlwn"></a>`DNLWN` | warning | stable | 'sigwin.nuttallwin' is not recommended. With appropriate code changes, use 'nuttallwin' instead. |
| <a id="dpnwn"></a>`DPNWN` | warning | stable | 'sigwin.parzenwin' is not recommended. With appropriate code changes, use 'parzenwin' instead. |
| <a id="drctwn"></a>`DRCTWN` | warning | stable | 'sigwin.rectwin' is not recommended. With appropriate code changes, use 'rectwin' instead. |
| <a id="dtylrwn"></a>`DTYLRWN` | warning | stable | 'sigwin.taylorwin' is not recommended. With appropriate code changes, use 'taylorwin' instead. |
| <a id="dtrng"></a>`DTRNG` | warning | stable | 'sigwin.triang' is not recommended. With appropriate code changes, use 'triang' instead. |
| <a id="dtkywn"></a>`DTKYWN` | warning | stable | 'sigwin.tukeywin' is not recommended. With appropriate code changes, use 'tukeywin' instead. |
| <a id="dburg"></a>`DBURG` | warning | stable | 'spectrum.burg' is not recommended. With appropriate code changes, use 'pburg' instead. |
| <a id="dcov"></a>`DCOV` | warning | stable | 'spectrum.cov' is not recommended. With appropriate code changes, use 'pcov' instead. |
| <a id="devctr"></a>`DEVCTR` | warning | stable | 'spectrum.eigenvector' is not recommended. With appropriate code changes, use 'peig' instead. |
| <a id="dmcov"></a>`DMCOV` | warning | stable | 'spectrum.mcov' is not recommended. With appropriate code changes, use 'pmcov' instead. |
| <a id="dmtm"></a>`DMTM` | warning | stable | 'spectrum.mtm' is not recommended. With appropriate code changes, use 'pmtm' instead. |
| <a id="dmusic"></a>`DMUSIC` | warning | stable | 'spectrum.music' is not recommended. With appropriate code changes, use 'pmusic' instead. |
| <a id="dprdgrm"></a>`DPRDGRM` | warning | stable | 'spectrum.periodogram' is not recommended. With appropriate code changes, use 'periodogram' instead. |
| <a id="dwelch"></a>`DWELCH` | warning | stable | 'spectrum.welch' is not recommended. With appropriate code changes, use 'pwelch' instead. |
| <a id="dyulear"></a>`DYULEAR` | warning | stable | 'spectrum.yulear' is not recommended. With appropriate code changes, use 'pyulear' instead. |
| <a id="simvariant"></a>`SIMVARIANT` | warning | stable | 'Simulink.Variant' is not recommended. Use 'Simulink.VariantExpression' instead, which is a direct replacement. |
| <a id="combnk"></a>`COMBNK` | warning | stable | 'combnk' is not recommended. With appropriate code changes, use 'nchoosek' instead. |
| <a id="nanmean"></a>`NANMEAN` | warning | stable | 'nanmean' is not recommended. With appropriate code changes, use 'mean' instead. |
| <a id="nanmedian"></a>`NANMEDIAN` | warning | stable | 'nanmedian' is not recommended. With appropriate code changes, use 'median' instead. |
| <a id="nanmax"></a>`NANMAX` | warning | stable | 'nanmax' is not recommended. With appropriate code changes, use 'max' instead. |
| <a id="nanmin"></a>`NANMIN` | warning | stable | 'nanmin' is not recommended. With appropriate code changes, use 'min' instead. |
| <a id="nanstd"></a>`NANSTD` | warning | stable | 'nanstd' is not recommended. With appropriate code changes, use 'std' instead. |
| <a id="nanvar"></a>`NANVAR` | warning | stable | 'nanvar' is not recommended. With appropriate code changes, use 'var' instead. |
| <a id="nancov"></a>`NANCOV` | warning | stable | 'nancov' is not recommended. With appropriate code changes, use 'cov' instead. |
| <a id="nansum"></a>`NANSUM` | warning | stable | 'nansum' is not recommended. With appropriate code changes, use 'sum' instead. |
| <a id="celldtset"></a>`CELLDTSET` | warning | stable | 'cell2dataset' is not recommended. With appropriate code changes, use 'cell2table' instead. |
| <a id="dtset"></a>`DTSET` | warning | stable | 'dataset' is not recommended. With appropriate code changes, use 'table' instead. |
| <a id="matdtset"></a>`MATDTSET` | warning | stable | 'mat2dataset' is not recommended. With appropriate code changes, use 'array2table' instead. |
| <a id="structdtset"></a>`STRUCTDTSET` | warning | stable | 'struct2dataset' is not recommended. With appropriate code changes, use 'struct2table' instead. |
| <a id="fstr"></a>`FSTR` | warning | stable | 'findstr' is not recommended. With appropriate code changes, use 'strfind' instead. |
| <a id="dstrrd"></a>`DSTRRD` | warning | stable | 'strread' is not recommended. With appropriate code changes, use 'textscan' instead. |
| <a id="dtxtrd"></a>`DTXTRD` | warning | stable | 'textread' is not recommended. With appropriate code changes, use 'textscan' instead. |
| <a id="subimgnr"></a>`SUBIMGNR` | warning | stable | 'subimage' is not recommended. With appropriate code changes, use 'imshow' instead. |
| <a id="urlwr"></a>`URLWR` | warning | stable | 'urlwrite' is not recommended. With appropriate code changes, use 'websave' instead. |
| <a id="urlrd"></a>`URLRD` | warning | stable | 'urlread' is not recommended. With appropriate code changes, use 'webread' or 'webwrite' instead. |
| <a id="vemat"></a>`VEMAT` | warning | stable | 'vec2mat' is not recommended. With appropriate code changes, use 'reshape' instead. |
| <a id="mdfrd"></a>`MDFRD` | warning | stable | 'read' method of 'mdf' class is not recommended. With appropriate code changes, use 'mdfRead' function instead. |
| <a id="mdfchl"></a>`MDFCHL` | warning | stable | 'channelList' method of 'mdf' class is not recommended. With appropriate code changes, use 'mdfChannelInfo' function instead. |
| <a id="mdfsva"></a>`MDFSVA` | warning | stable | 'saveAttachment' method of 'mdf' class is not recommended. With appropriate code changes, use 'mdfSaveAttachment' function instead. |
| <a id="minelv"></a>`MINELV` | warning | stable | 'MinElevationAngle' is not recommended. Use 'MaskElevationAngle' instead, which is a direct replacement when specified as scalar values. Row vector values must be transposed. |
| <a id="blchid"></a>`BLCHID` | warning | stable | Specifying the channel ID as the second positional argument in 'blfread' is not recommended. With appropriate code changes, use the 'ChannelID' name-value argument instead. |

## Automatic fixes

The entries whose replacement is a plain rename of the callee
(`renames_call = true` in the data file; MATLAB marks these
auto-fixable) carry a fix that rewrites the callee token and nothing
else: `gop(@plus, x)` becomes `spmdReduce(@plus, x)` (DGOP),
`labindex:numlabs:n` becomes `spmdIndex:spmdSize:n` (DLABINDEX,
DNUMLABS), `caxis(limits)` becomes `clim(limits)` (CAXIS), `isdir(p)`
becomes `isfolder(p)` (ISDIR), and `Simulink.Variant('a')` becomes
`Simulink.VariantExpression('a')` (SIMVARIANT), all byte-identical to
MATLAB's own fix. The rename is behaviour-preserving in the current
release (the new name is the old function's new spelling), the name is
already known not to be shadowed, and so `--fix` applies it. The renames of
commands and handles carry no fix: MATLAB's fix for them is not probed.

Entries whose replacement is a rewrite of an argument or of the call
(`rewrite` in the data file) write the text MATLAB's fix writes, on every
fixture and probed shape:

- `string('abc')` → `"abc"` (STRQUOT, safe: a literal single-quoted
  argument only, `''` becomes `'` and `"` is doubled; blanks inside the
  parentheses stay) and `format` → `format("default")` (FORMATNOI, safe:
  the documented meaning of the bare command).
- Unsafe, needing `--unsafe-fixes`: `datetime(now, 'ConvertFrom',
  'datenum')` → `datetime("now")` (TNOW2); `hold all` / `hold('all')` →
  `hold on` / `hold('on')` (HOLDALL; the two agree in R2026b, not in
  releases before R2014b); `-opengl` → `-image` (PRTOG) and `-painters`
  → `-vector` (PRTPT), the argument only, in the command and call form
  (the renderers are not the same program).

On each `function_call` or `command` node the engine extracts the function
name and performs an O(1) HashMap lookup, reporting the first matching
entry for that name (some names map to several check IDs, e.g.
`maketform` → MTFA1, MTFA2, MTFP1, MTFP2, MTFB). Entries with an empty
`function_name` are "generic" checks that cannot be matched by name alone
(they require AST pattern matching) and are skipped during the
lookup-based check.

## Examples

### Incorrect

```matlab
data = csvread('data.csv');   % CSVRD — use readmatrix instead
if isdir(folder)              % ISDIR — use isfolder instead
    disp('folder exists');
end
```

### Correct

```matlab
data = readmatrix('data.csv');
if isfolder(folder)
    disp('folder exists');
end
```

## Configuration

```toml
[lint.categories]
suggested-improvements = "info"

[lint.rules]
SUGGESTED_IMPROVEMENTS = "off"
```

See [Configuration](../configuration.md) for the full parameter list and [rules.md](../rules.md) for the complete rule inventory.
