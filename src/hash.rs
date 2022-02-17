use crate::engine_constants::MAX_HASH_ENTRIES;
use crate::get_and_unset_lsb;
use crate::move_constants::EN_PASSANT_NOT_AVAILABLE;
use crate::types::{Bitboard, BLACK, HashIndex, HashLock, Mover, Position, Square, WHITE};

pub const ZOBRIST_PIECE_INDEX_PAWN: usize = 0;
pub const ZOBRIST_PIECE_INDEX_KNIGHT: usize = 1;
pub const ZOBRIST_PIECE_INDEX_ROOK: usize = 2;
pub const ZOBRIST_PIECE_INDEX_BISHOP: usize = 3;
pub const ZOBRIST_PIECE_INDEX_KING: usize = 4;
pub const ZOBRIST_PIECE_INDEX_QUEEN : usize = 5;

pub const ZOBRIST_KEYS_PIECES: [[[HashLock; 64]; 6]; 2] = [
    [
        [294304103279159137451972492103566810045,82183680145403513760099524767995691189,158596680598802628186995910085078697888,169719051326004253466983880632766201662,180795236706374721190336964162452159526,80930961783762193681346703060269013799,898496373097266884075387782478188455,268895182909858382810290496965063274669,252791441925733276639414542285506746674,89091299317125577166452213902246700597,94561784708865115012008167066670794929,228242844023989268950150422728108722511,45035454803500235428031690441690740731,245643073901475560323806754217136239144,300923723330574263112370906294290108763,32831859195570155992776626383748761782,265774672813480420453529347047024012058,227761470473116942441357606692679627458,276596193965388969610461186162820099916,181300577715121170676845350740955598739,179871877255438337325671478948486465310,264294388987728155176613327448768979087,281122403712236219807402401142348513720,73290864015325526895016110964794039965,21921342584488549528770362770566042706,129597947855358217509954550645244520319,338349935273060860472898278298799380170,319588592882740328347317178005900400109,260665149759284204699089349623973326291,162080917335966987332759124795286034111,207819980918088469879810799722188947044,252916047785513272631074919017819700223,143660956669271589225330780215124773539,166536084375062401861707322348759285942,184816974605003014558189839893808234853,122218447238055759836794329961079400109,239488858038727295953254319639713952122,155388930971963169338886009875222709219,73622871171678176441832085004296536533,274623705813938924024575598740947586047,283411770480433227580722173636400552291,199999253642425830714529778441791285234,112856844869361212254463602943305246142,326385271110662317902509162268293178802,105822864792168731604517072631679746620,185378089860717416668318401083585121359,82467597509979195747318243273239628663,36124834105103644784235139432921783469,305613550983555339217955808723030444294,245338697413507091003425091259999421749,26291881561448180675339088373393495967,309583176147718332316322234967512461130,254599748099945708917068272331688536212,49903050957873657134292733971139825771,52542712931808304419547345284727047411,10976487017702490737521721659692944807,248155363174684330314627729439891503596,177811045939227528454012507214233873698,308420071856058298384818971866251252409,1063151797929727683377386731951458345,162534279080050005498228841065757691459,67784819427205065163758514357327499425,195127228606382559048296889031577690198,117169171578956788673139185606139902986],
        [266347248980625042502300689663412995712,312713953788030921759732534330850017033,43375615833975650550502465763763967855,17783019240149354175986433911906043410,263460353153479575543210080623500532741,298734859649941251491420987165254535195,156235143111419863386857676261157499816,218340413447027673466860565438704823927,24582380121898929926937875917928545311,175064502812355954249797392131989929110,216046595766345473938893414686749136019,190276894189604591482452265323640324119,8359633981328367714184805662012819502,174575948184625106157024350675173929369,123045722510620531567592055090898620154,18892402868419729149697889367375683283,63732790924896612837542386369554634076,59921650630479425405257292558594166021,277834204843037235286814485245020182615,81824792591096158666215690062204412745,266960993428622511713983915726293716333,78128600781848490552723475702180313809,195979267534449528008002385130120105028,42456109349982106370670668687578213046,97205238897658487642536444766399368399,110724284752245830316896676059459589708,265189137618939699806966229394220405255,190371696328687364443217695358518813332,161707974960679289579696978692544430798,125720439570608774555796115964413322223,302455911262570574678736135127482284857,106039389926060589133959748317612651166,91372546253449576349094750989218674770,234144678061785471738610306195188756736,210259560409815784829305446213179843904,45533453199748206677470010014013987363,307534373099121197804177123299051807459,56709974037671752322666715685132483636,230773942266688182071041132191533659745,59621109093587085320173977653138315014,112489728855296722494636558074931788885,71669510400284649593126778110053020593,50245966988758775254371442637102602786,34800105384016473026875191157218054327,263907047262406465035324391572828458373,285752562605321335503689901673667939758,291149609966726896955848168912714770961,170897886050445739259717736658851816260,76082629513945784376874564742736007056,93771298341239053680463245356862131645,74340065718763377176001594129990641135,191913981352096436350650445217683595655,144080570429224941770552750174101642656,140487322635013974357363032499564084005,46104420102689242036850385100479678492,275096296701992767836472807330326007552,305488284553447368180606505201028228917,273028274440843574741618178009903606354,206808242571527415735730890936997853326,337508273266209490063364630095060963162,119505159493340347894476038375020403238,263692620336659295417850343499974107694,15187611268860296784873386969275737067,160592785820175986070021778718589804183],
        [247548938911628764631981694604170221938,331400810922008692031942500665188368926,187393811437634456733028425485095426608,177034836433442573568234446936938848340,284025344248984883533224598382843762027,57730924996983056406680102458053418549,235196001726048675210982567070274102654,203623285651164367937154982443036337104,270161586131746689567326851289185027832,257744015473273358770924544461451879790,279664861342082614349278540859846347150,196777705914523730634068398128235189555,281742632613262832294542380406524485253,6440736138227476088077076287877047935,139850906034355496997055697965829063954,50003579043937605041663963516005514373,224925278082910933383887224798614407433,275473723600927837062997791345949998544,261115460224017242395935979607640709220,114213864689733018117963284256762801013,115002566908076670668429227436875216394,313627205354159334458807087526582874509,310524013417697937304684629003655865779,221410958408074532601764420734989294393,74985614913944403610040250947877174891,191543347305289337662815616285930822222,105549883382614689786175301773819892545,246538871062023868575119223216260100744,80707311088743423210451772756603030047,331531667584934094757822437734828295962,87627655008853649480403808268540914615,285237890305491663262621435708435843815,135669235482674048160752623513333507848,311546952692786853598173616853976764152,151573864288122667903504109351485218266,26789294224361051270206936625898349534,98286062799092798612819887429752746868,315406668931673756651187358751019405414,49519473412545364950457278989294912080,324741896715479655012490065788757701075,230734961149129424870293450142081327439,26299800512406635860157313844412028188,205207060239728091951579106753239384111,319970101294750541666956218243021621426,125785069334077324508870470164365159108,87207444467187303212273796519752554017,276615642488103322185621861210027555379,56242039372822963479564141025692340495,323493866361721741175182810921023621680,320402330325961111175450906997080271949,176784934902135216794081658229781615950,187671257677630438010916123778447612721,285214188680475702258797537428248724604,208871209858461380428713332970855788101,160659023958414530848307724523806698368,127185750881511197025249843059550604220,66678698709147324427171384469539427974,255507053333350056686150462492208498790,71013857761813558863938955804115324991,199679363785978000406314447892014296250,7391405504178482163255438028187894463,328766367492493891873646005987119421594,190038633694553984914189464070537344266,266848418831702792421247193408982497753],
        [244745164257584502206910837962188961788,235629561855646822639256494016509781967,72519461870860957741790297962941907091,32340463184069979701647901611030671435,119374987653484668368865107795860296695,74515730373345522732011869416387387473,93274657770574807526233458787921364826,142958256484936347583635635543818713081,322412245621368890343614079187944671239,218064180991324781502377476059149703001,18069708848734686221492304267780164269,16918968424399710908928613123360508985,26858376951583697320028654454472839228,230934433632149577906461947089753202115,40434625299481423238081190703385316898,20985206113340496278148203226143201072,253610909071158291812269690148603316405,45514071275159296599906994413770611132,248426087943383401149383376261089198072,265190041718505746253027618295963698340,85336068689363535932509904822371258924,94985318611742203345849775110775766317,154416904816571983766316011867069642111,330595175529081219426196346026160418475,169139564363288126236491729748422060769,133061246743495154760083854111385486922,4629233038446865848761199309817466810,64046209009442077452347286802900947100,227572729360099959053923140104191365111,34090535487308618610500776919946343739,89298452412852202737504840145553255325,278006201806565546873830175281401747405,321565893800922091063993008571332077274,902779325126286437037841881330109291,164430947954136719822420003102207504619,134678822273740163342943312879167603660,85885792887001498460693968950148958320,42887842527471386577026087492746965956,303707516185351336153852096209598662010,174703965429779321480431295961475828577,199092431855209730902053167854140754273,166245051473359893725592616253039143486,210099212820824839802642944111520594434,150227658730690210251398211224840398645,156981113980201692216823812418471303325,63988357410257694164601514346773162906,5571275937596070278441892287389708974,84631788026069302940469593771225146150,53389961069093035089567567116217681353,12091850198605834454349426342319841044,276596538713654758886148118839582854814,73457904085689096141949863358016288505,83219659053640760559087726326992870202,15480120643393039911826160828349616697,174280919702437543715863374939417039306,117470702733897871089608094526196951198,169014869993682353661046498129029751438,205310946959456789163284609911833596029,46912681179077003542533090867871014891,317039397510212813424503910099940443819,113150713011132327237752880492928873923,291334661707176364640638085564628668327,233684603174120592109324307406740174555,221169096570483334828843181503888487406,],
        [192663716421451881615760543373519662257,285730335543319432584609184219128832796,322729037535327824288794451835557898672,311764270133864359129751762983936807745,95322723705610609993789476439230939032,136587331240925684308383409451684733134,81367702830986572216799313108437008147,71937670491348234961564032093986814008,146300160946606136458921187276418043786,74137214607542188896941652336648106868,128589389424973099711692810157579117829,207664214687314323696263094451807745500,114947686610527932811195396633317668466,25011650921724145347241965375091818770,140381864304402427304081086541754512257,19512779413096152494612685732994312678,330696292662741823663667648454458185844,137364335662321694674072740216742921462,269052991609399537664945674536344350992,84385291924463061878773270577552333834,270305516822577698835210688776561843879,227120160260295704478909703758272026741,190157921050854157927257530081929671070,6665955110118132929527030417685430340,337966874955672247181600647463087879132,93671751489362005214801801378339345341,244143480001431649120347072385244860178,239853333288639127183981976380070339974,191399984998985591115368589503416466309,239572969185945829251055512694716444642,133707405824459900404944036071908775921,108255183163434662994461145461452859050,99909834186201811965463722854605733140,30827918833338643606528330922808237059,217429020054496273294704650537792418637,139074502699636830321766133006895574742,34657015743675826123794458831252283612,213983873246191855826088230599278211407,140454255557801530914340656897818224657,9200979108074320396824693197838715226,280334139370206375384800154569747141197,27462982585308263139419000382358042614,2943619628517027069601339593319130748,220668879183900821864609976815110897659,326492510650375067254362312814098133080,94727709667977794663123130304992420050,206592821874231646203872066773469559532,273754180650512597088954867533652616076,184210885061789874631089258290436350069,225671665813486578486040938743396794018,310487441294747181037293154869239558747,318159910247758311146463340595291936376,150000935460692439306080058686232743986,318803822641141969675332710738051483983,250641867635659656855997860919801607276,100771058365639001748006482854744223645,159632540648628823343478549178677132444,114445385219053225529195146622274419730,159661482395556077598762047783008914123,34473011504764608593593765459185827494,156368515641620960368468358296810241651,318651501957112297528290417779394007772,9111966330957555740888883998827793319,241387631084404436249674222095215135250,],
        [29732155054211391458999446881197909525,77428482978413237156543918634695872044,244366889951926604529440683997320147820,306401135761959594697584686853322441522,27843830804709630392844786090012393532,183360361853409981490605833518368911088,312577362685597625440700797214860726430,111410407031111776619069911376834778909,99284847795050115188664493350898933975,313440432144968332614763273562453625579,67317998742553586765253204949361105309,89141802650920641959959187775149959075,267854584971552605410858996025057777211,106159182199072411503923451457732814426,23481653031526619691511638525405924421,33036720532707258320699690595554369603,187224583647173848907350454354104714625,55311233168709573038347748559180122590,275829944614802262402757833043255551317,17815996053089332861194471549080116938,167012621110750519478939240111216684933,46591404637629341172527132851688420977,321236457686197646916074911436740177031,300972632567974545240742194800417805625,286275790410734262002470058468621970236,221309847240853874311883707716261077157,264526046521739884447382622275419695043,238034871650118191264822083366347459954,71827038223414799491252667123184893672,78182980225349912364158121527590560066,245300323850250243995733067519895409914,217475373730425443348078902969888024594,332766030106230984440278669529674120260,140945091170107071727459602071838931167,187149290043359843103411546163241640993,241077521150327401315245538300281399951,251016688619924656987042836315292352466,218992210486782403913016291264042395418,199546982834605487760054333612610948604,239551280026879201201592800850243510998,50523508086212646125542289991851977281,52922236245354151570070873299113127659,82015769640127855791416519401365304976,85980885076188984189690923053059535978,264039863352297746119504509915642015870,123886738411361424007490883303663589682,95974672261813993121491647971533456293,108198946952873362398746266938981187893,276716311678761246635997006194003232377,211475672653582414285873196505454083929,70315391824893594643911589714978520868,311113687250308899881257270530367505790,307149802984853864671379834537546779820,193211844607929181893091951533638459089,128906869519094845193695088374459478593,109297934801788012270576944788917600048,30685981946396904124459424802443064987,259674774899288381867411148116648693655,293505828258403182425342579877687567880,177543520373332568507261923460061173364,103026165896885813069700941531240225905,54719680547214807376204803674164928831,136981488786836902701000817524429010254,69787566143741380683506085744324151091,],    ],
    [
        [115029559284067683506991492457826456851,285533077915020418210053905334904929771,30709321867009388879574420523484932908,317962950614665514697549216052448059234,157069170642465241639545134712536618092,249615676652465822509953138242168061279,148868396577551277620160490129862094366,182043932723663302028779114850459548263,326287026736223846638784034173720138115,149231254805353530472902369119911661617,153484517554227032860632432395604416562,160558930751443542600994469312282493609,260533860464299512258601259432618122056,229633958290501531557546055155415134126,46896588866520524109956727989598901649,68707927054789671402784910407981877068,55746643530278894187533286745209551896,317521902705929409751095696145387142816,49342671731522895827217847527750486672,228101928303336475295913637949167597149,266674674048629385808209103118038747647,32687804976555537105316619558943381887,77850102204437660599376879148439480334,17635395184316113683292136311948193652,104998829064021824170065355701950888520,309224361596841466401578407464344296063,176021656003717932810006082702051773346,19171451977847120949193861020538157228,165232634931176504046329565494063310624,11354071932030565663673284713110600607,78136991317370693294599357228592446219,231091601826222616476893493337754894489,303283635473246588209232386923210432111,38896625086646817958729448647661314632,287505218214042836501811850338181573376,162242806947263215360536990133581264480,112213625571815199194760506532907363670,332112892745561396255064555112250161171,27489328657130417733445612738347902284,178695021835853756086046219674433320946,113646695611032110912204753622052040221,232136835719123765385000305339751357069,268515365710011094143145676532023430888,238217148433068951426699300277206054132,223948795019451274784761793852995751931,66581128028704618812270189270474418413,201937074725982651841608515266341426108,85794610741146800616758780588289290769,227367923075074867320722292213101737268,286241227815983835051505007440088867196,293710059864929403964778210174857748258,84660775178999192203303917673796328411,49529209291765281283423306579354891046,78305514499971141067782344240508178217,68647019434266146051932662199486906379,141280792625487031640295184906163149631,204089048476563798131752250782570446469,235874463344377010085394492518529921671,143628058407594442813092780359238695938,268639610993145962359375259355027924061,154443950612104373436297900779757195543,271692300660378664530855413090162847954,326057812821276849343386674547039996141,306530885002048470700436375661590977056,],
        [190517302722173780609655590198177547446,200775292715600278764543642301208367686,300869036242242184276451428904138746058,292916052077998950510981766684101420447,254063318253123654271191715454958972811,183734903037473475388772458974664841963,45058998516766085650776032124381063353,123447791464154270034224092252808700491,285592454580830684253955755877102090930,222887213177268700643905333266192867013,74828592896543614354241580769319368480,220384869693264231475915875809492720261,277755616330511821984940987114323352725,272471956552477311757692614213075610564,316429930710564397257598550840928844432,309677486392486346399849048549685019360,207726663866971136056673503817884858570,247153746386217771328326985753258053307,38303301536854442454022361032247331793,232576772458097280201858377793638975541,116920935213670727603994777886561409300,7084978629758140710131192294331567299,86037422526509686202001884977512441598,42656594751073400003787423078619158675,22921190741152672232738115886657834334,233739713412268452800062722299528067707,21065125008816797036628679263308145365,295014861987094252360616774014226168988,283121087650438187773436495164766766386,253579294201498567809676885848992352367,94192692301529377786690451381575399440,15542178986719361891517165266632254240,33236786084924152228008117817917332277,12843160230054233915218498975188179351,273563832116492656498041754011399068460,336807013480384244445916540806232509962,320035221301143197378756162439588961351,1665093886847273464505647425232932388,93092465583726736396403156844010564709,103338043857597493315539581941221738808,149003711996000804542617868605925513587,143769583632794223470981778938086886541,125413641465160409848767862701095634782,44148863287016031629990565164445726238,134914241940071015884007266089657349685,311710247988991924785163651739370398599,132066711531384799431634181855259964437,266104691766552924381252436820060178069,206923684093609415941239473576378016815,153968086301178680095875258973669225185,38061311447793281418028160063490042143,132370903360500379639005748567556436372,180316655689137291249368427574530854615,56674574217256814478859708233957054765,314556290331127314609897870519021258998,302754207723165800188639894437724655129,78577862972954284435238482983357986616,83013786414897379237591692404593988196,142545021592347172868764329041138024879,16436032649664279436231614067054204523,269359975371506876710529165608916258501,62172922063062930169311700009371221459,182401840290998658531519433615352953188,224493708247278968431497031281821505017,],
        [20160696958916386479974133520731805328,151908761372184458170583598036262991282,193322110498229867374542601470174673017,300712746119425314213158187506111312388,202990437129998546780315440784907808936,97588829317299549143487900448441181232,173250585025813776722222831130167765040,50872490366612938598010510709485415628,318120479689391294968299066445504897696,230151692384684025511801986798035426769,231412160174669567972708970440162489226,18810193587450167877359316113934895970,310624367697902404932522207726512258054,132831497381450477010468740773886072310,168220717929307887625564694955108601998,96030803158873199642135853465554267319,43271270173472478828988129886543942203,167039811707056588070684265499265639515,294394093189872803495793928342006888260,264213240598014365359967656991723768244,308275762516990820321038145346236189852,4743610811946775867205892823938757520,151808281263280996119993859184839629960,300098277101167548585937445914299877204,329596954466911036011116549431257213657,25329347147538940549317332923286207024,123551605432616412809209063771592229639,67459011903918870607820027477481787329,4545374969108881643741625320934909335,129789328679440528219626302902837260139,294950037410710759728236310402807796314,20809799823832969489315607978163035300,263748697160336945460790282861585514881,154445118850855593240484246797837024461,64441550048560543747914758010591548999,114626226526063008711640469254358103662,118817327867795232419211281543963874177,38277114277215947564423612378958674919,257617777130223346268417982923640024152,106046987228741587540908666858113527001,315226794820082807802115790951508116978,229962445198790184109394853038265299172,196582157868415245236081665019261463865,255531777775974505580628132933366254857,254247630368456387188586099800937930374,162152952614722790289488467824405285250,142184204075282548673523290391883385567,19886966108883374155570892274272923167,270676579801746333718565628526634325180,295449686357916313587885200589222550345,190432243392875975430888153642361496002,200981482158455344582067546086010786287,23306527008392598468247489291960489700,62857471197981196853567895917129055665,239980565380390545560939181460641767812,324129674555030054558364337827757749758,288418759531642308478689641719942103804,206853717575545583596684724660146229910,152459674493910090515034855342357582027,4782075471460553628527202029102043221,295608972962477017568141404155184865005,246964780104401329139658883089051576466,51125720691172278462981168921029512390,19084231341231528220609915904327462166,],
        [295796189843082299675143589707045030043,174388730574024296335331614944047709880,309696928158473324861405069392869547008,57113628820351585644885867017217696857,145782527762207166140601195137049617483,315308809038453258136406822545380672607,303841185765301232643247447160147158015,198395484725576997256559788658435730502,157925507381861820750008289384441844112,217810746563364110350002842214332819598,185520560633973970791330682806677697982,192704110311857361169538868360070634488,212289018174604543148777082814009972563,31024097002420272644633274843187302021,217737319171056674730939896501764270404,13690068846456407331809167055513681882,240088740065748822696392909513742658969,79840265983342485723520247207963363182,76936522807441088244557481468299310571,249463929099322413377445625864679150889,282044536446396758595584721987824959557,62460692768457909654560615458806334545,208337946390419577355504834478416033310,20683504250988184886790237869849279657,85288864306699257539863998462515791864,22627784590848696305331555018959726631,31062545052053812853817550373134097533,74242268815965442298014990045441834443,244879912142980161281466695639108946099,134451066629011083700946418937870729414,274095643609594994461732707882749784709,50444710503166403510094148621999954987,243832227661371668406602484983526962033,143236724316798575638521110355960646289,333160483248978611279497008804481064855,205525682731179736690254170723612867947,184232461336403835451224203181699145575,84360912879743958532588430050594026735,101349522800148205768302188279034102262,148969333828633811791041076680846654334,66821495848150775040617389624742630332,1761940234046832334136988824218119938,142003680816102684138598590772713025538,208586045595587570402470790619386581742,54376740595200627755726519476794871464,153618669346362065426095405490379791620,232903628516496852163892456374020874070,111690029310177946423403270963389523268,71710153044257996663387669366210046969,21695760530789409252071974599089223018,306599378821087132229960640686208183563,323723413067302860422615986454827302890,130617897948897079798420299433181289966,90581757308112123860879702297839339128,319503504871446649811199795590556483093,91680992280766292512688255004854507763,210860485499370518225760093432075769937,17995562495720721800513053001030904779,333868287546441824074989391149129218675,10711242872666848065296865304150226049,192091947923049432197697273940796006060,242799237211656055424525518451604512131,133706723587436766930670874273373886982,151293719214778425389398233077951180752,],
        [164439566198124065160535972831773349080,131342647336716644781223183906203597157,21650425612082992090181325416990556838,289378477710841281615127263221637853916,193930406545518129824052595441418735188,213038663532637749612238768790883572677,315023467660992058485999159318589218164,290240095049277178382156195849982405911,291226025051420920491154651792685581434,50598651457335622987263760897515013671,57948476525962081272920345229445894051,88888000982898543566902759715671436938,289473759057535018466525211020577152229,57128398562345166885506571619349214969,322741501290699306121512136977158831860,10334284226890059528223520227891780894,55419983870511938498156756727696172716,297360998711811265032147929665862256268,263176879524948058029085725282815849623,110099346913968734497854220494313405106,104234746978670581242247111460759167574,128037033436932367047989205142488604775,122984061003430675489184859108618821438,247069496625219733948478341068664965223,234031135381494670844618908656236173446,92720249224686587372779972539298266813,146219831843421633200333565582687376199,11529680345452320292912756405125020727,95101318721852471358991945483304309580,210247722044533102277760998488607273867,9880290488054303137685758554892132472,11690876657793401867978846903132428362,124707494618622633470466431504617535387,313452364559741279828701587503772193592,175042445600994195490935482456613235444,6718006968524468269754500333215468168,89169747980661351993847640435263958371,88040032006911173753939004224958729628,329204063473109553583173064552925627322,50714416875753584548470416037890449448,214549229101653137105322457502966429351,129822608958639228022860113109576129978,328309715568606879563465778156163608972,145463215547100230965433893340733056808,8559246266701846005431637293005295677,68015855646916375067064929227981911677,158692230146495558270614042805466260719,79045456038174543369611124876272659373,28646050194917257153562539106398213819,258478051763359859602894034080631885919,282950345674484768965954317110446086538,271654190067233811223356593302665693106,308237991810451752071820254789552326368,253700281224356635188607186278655163105,318812574465613806104917433622491468296,236426738477926533709996695231834796322,197054992604065469793887956835841302502,158187190634472372018057634028943430338,155861056302070568820321692433926622635,217918644333104134222982332884326399050,128731795809126693634274261810994021562,153217760258875317607990013615239151802,67699133732654393793611548226331118399,124494654817279035754435472508818350338,],
        [70699594045997968183504165897048863045,61650454026431057066022862537004472446,37995831445030930436648891326327651284,174165801129864154338540110351623613729,321335685119792911695132438692481208514,172011475303889475794938690451308252632,22998756551087818313353381699895643928,328056036590912775751852441030509487579,35482828114926673264434289954026223714,153956988969654749512105032295893745108,83909962192205657386122331992506823897,44633186030393203724711753723349937896,72305562836241513904413452449180622011,271987466999422071045255350760852936461,270526871430953367931880009313621130842,127893417031289047760529977480266158785,217589857121614392211439029386890422255,244400798671030399104918249995209020267,255822511890874096321607177538455806268,65989067195257571634274360305283178007,283684078203478179406700958368605335218,284935380744404878754351118640015348268,4956237498027567925983962989059648005,179266203640854699423384435605863345031,283327004061565430217220230113850685292,70330858049533030562623763739953949332,81093524494662782653028658143737942313,62816925111792958499745687548740822744,49976710140665503052326513909005274544,325308747929282238633638458546238977231,182737979851802366796474984711032124092,243719891024433391282590319108694116413,132562807972249983393787500907346573582,27723638787192025466639298393815732597,82875800799012896179410502896690193310,157918936770392720198445783632941496233,22101762975406502679755115255374124621,187810402337040403804895313223060473085,286039454869318726914707660268290193187,246851841411872823517958681539613985193,25887678880892807410906272434629973971,60400187351301003771947724938405062228,30420753281937268426837631302050928763,209158753600310418387632096485749110908,68649560170787335852235656374301197176,51780434751466560726101337544431091940,190289738752200136973318078070839685817,293188585896802791368726339845511753341,251595288321014146266358118657746360122,128072795516849184083160109238112894322,219469912991120971166012793368673379733,156258705267479666252201711659871493954,257781902033330218625877789284312478552,107363255841910428162640997038991857611,300046295623419592630878218731458811812,322992740311549464685008159312830958179,15337738295100649952702694715546217117,265853202228827338681011501235054817260,281037492214026291808191339979559182249,93030818295299091815377723408422720033,71145431309240057831248144484340946231,305184808684912733049377642098734964269,79534996285304111754816720718377457230,191496820967078393016339844866326619941,],    ]
];

pub const ZOBRIST_KEYS_EN_PASSANT: [HashLock; 8] = [110406958431129947971260781754150585982,154925975523856810324882009768310049951,90450030808565630129711418597594075278,199639618743211216524324144173310730188,54709443072887130334688870554290231782,106512319587626975155607028958695526977,267039191507978395065881970224868660012,2280422199076206247951981492776627267];
pub const ZOBRIST_KEYS_CASTLE: [HashLock; 16] = [121119492155004654471420530365384370918,132101411200825162190627955664943261921,15604659435512924620742965129942600631,231059104177204803040376598701508065714,314431097251705706033873739423765163361,55599668894412637576046114527814887378,128352032414728256394026217842396294960,243147834912439324996826201452394732463,116673560559728937715543340203418822654,19628170522283444613775415049927768542,16435835997888217138109891517880930317,96554102363108411335529388486063536139,109170992500094219938817747464695088699,137775020524864871991823286546251979998,169622693462175693799350213289755778212,251214131385498871743329564801407975357];
pub const ZOBRIST_KEY_MOVER_SWITCH: HashLock = 13805100923890336541896468202157340825;

#[inline(always)]
pub fn zobrist_index(lock: HashLock) -> HashIndex {
    (lock % MAX_HASH_ENTRIES as HashLock) as HashIndex
}

#[inline(always)]
pub fn en_passant_zobrist_key_index(ep: i8) -> usize {
    (ep % 8) as usize
}

pub fn zobrist_lock(position: &Position) -> HashLock {

    let mut index=
        zobrist_piece(position.pieces[WHITE as usize].pawn_bitboard, WHITE, ZOBRIST_PIECE_INDEX_PAWN) ^
            zobrist_piece(position.pieces[WHITE as usize].knight_bitboard, WHITE, ZOBRIST_PIECE_INDEX_KNIGHT) ^
            zobrist_piece(position.pieces[WHITE as usize].bishop_bitboard, WHITE, ZOBRIST_PIECE_INDEX_BISHOP) ^
            zobrist_piece(position.pieces[WHITE as usize].rook_bitboard, WHITE, ZOBRIST_PIECE_INDEX_ROOK) ^
            zobrist_piece(position.pieces[WHITE as usize].queen_bitboard, WHITE, ZOBRIST_PIECE_INDEX_QUEEN) ^
            ZOBRIST_KEYS_PIECES[WHITE as usize][ZOBRIST_PIECE_INDEX_KING][position.pieces[WHITE as usize].king_square as usize] ^
            zobrist_piece(position.pieces[BLACK as usize].pawn_bitboard, BLACK, ZOBRIST_PIECE_INDEX_PAWN) ^
            zobrist_piece(position.pieces[BLACK as usize].knight_bitboard, BLACK, ZOBRIST_PIECE_INDEX_KNIGHT) ^
            zobrist_piece(position.pieces[BLACK as usize].bishop_bitboard, BLACK, ZOBRIST_PIECE_INDEX_BISHOP) ^
            zobrist_piece(position.pieces[BLACK as usize].rook_bitboard, BLACK, ZOBRIST_PIECE_INDEX_ROOK) ^
            zobrist_piece(position.pieces[BLACK as usize].queen_bitboard, BLACK, ZOBRIST_PIECE_INDEX_QUEEN) ^
            ZOBRIST_KEYS_PIECES[BLACK as usize][ZOBRIST_PIECE_INDEX_KING][position.pieces[BLACK as usize].king_square as usize];

    index ^= ZOBRIST_KEYS_CASTLE[position.castle_flags as usize];

    if position.en_passant_square != EN_PASSANT_NOT_AVAILABLE {
        index ^= ZOBRIST_KEYS_EN_PASSANT[en_passant_zobrist_key_index(position.en_passant_square) as usize];
    }

    if position.mover == BLACK {
        index ^= ZOBRIST_KEY_MOVER_SWITCH;
    }

    index
}

fn zobrist_piece(mut bb: Bitboard, colour: Mover, piece_index: usize) -> HashLock {
    let mut index: HashLock = 0;
    while bb != 0 {
        let square = get_and_unset_lsb!(bb);
        index ^= ZOBRIST_KEYS_PIECES[colour as usize][piece_index][square as usize];
    }
    index
}