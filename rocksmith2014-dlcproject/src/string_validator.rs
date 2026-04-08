use std::collections::HashSet;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

static USABLE_CHARS: OnceLock<HashSet<char>> = OnceLock::new();

fn usable_chars() -> &'static HashSet<char> {
    USABLE_CHARS.get_or_init(|| {
        // All characters included in the fonts used in learn-a-song and guitarcade.
        // Matches the SearchValues set in the .NET StringValidator.
        let base: &str = concat!(
            " !\"#$%&'()*+,-./0123456789:;<=>?@",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`",
            "abcdefghijklmnopqrstuvwxyz{|}~",
            // Latin-1 Supplement (¡ through ÿ, skipping soft hyphen U+00AD)
            "¡¢£¤¥¦§¨©ª«¬\u{00AE}¯°±²³´µ¶·¸¹º»¼½¾¿",
            "ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞß",
            "àáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿ",
            // Latin Extended-A subset used in the game font
            "ĞğİıĲĳŁłŒœŞşŠšŸŽž",
            // Latin Extended-B: ƒ (florin)
            "ƒ",
            // General Punctuation / Currency / Letterlike
            "\u{2013}\u{2014}", // en dash, em dash
            "\u{2018}\u{2019}", // left/right single quotation marks
            "\u{201A}",         // single low-9 quotation mark
            "\u{201C}\u{201D}", // left/right double quotation marks
            "\u{201E}",         // double low-9 quotation mark
            "\u{2020}\u{2021}", // dagger, double dagger
            "\u{2022}",         // bullet
            "\u{2026}",         // horizontal ellipsis
            "\u{2030}",         // per mille sign
            "\u{2039}\u{203A}", // single guillemets
            "\u{2044}",         // fraction slash
            "\u{20AC}",         // euro sign
            "\u{2117}",         // sound recording copyright
            "\u{2122}",         // trade mark sign
            // Geometric shapes / music symbols
            "\u{25A1}\u{25B3}\u{25CB}", // white square, triangle, circle
            "\u{266D}\u{266F}",         // music flat/sharp
        );
        let mut set: HashSet<char> = base.chars().collect();

        // Japanese/CJK characters used in the game font.
        // This is the exact set from the .NET StringValidator.SearchValues string;
        // it excludes rare CJK ideographs (e.g. 糨 U+7CE8) that are not in the font.
        // Full-width space (U+3000) + CJK punctuation, kana, and the specific kanji
        // present in the Rocksmith font, plus full-width ASCII variants and half-width
        // katakana forms at the end.
        let jp: &str = concat!(
            "\u{3000}",       // ideographic space
            // CJK/kana punctuation, hiragana, katakana, and selected kanji
            "、。々「」『』【】〒〓〔〕〝〟",
            "ぁあぃいぅうぇえぉおかがきぎくぐけげこごさざしじすずせぜそぞ",
            "ただちぢっつづてでとどなにぬねのはばぱひびぴふぶぷへべぺほぼぽ",
            "まみむめもゃやゅゆょよらりるれろゎわゐゑをん゛゜ゝゞ",
            "ァアィイゥウェエォオカガキギクグケゲコゴサザシジスズセゼソゾ",
            "タダチヂッツヅテデトドナニヌネノハバパヒビピフブプヘベペホボポ",
            "マミムメモャヤュユョヨラリルレロヮワヰヱヲンヴヵヶ・ーヽヾ",
            "一丁七万丈三上下不与丑且世丘丙丞両並中串丸丹主乃久之乍乎乏乗乙九乞也乱乳乾亀了予争事二云互五井亘亙些亜亡交亥亦亨享京亭亮",
            "人什仁仇今介仏仔仕他付仙仝代令以仮仰仲件任企伊伍伎伏伐休会伝伯伴伶伸伺似伽佃但位低住佐佑体何余作佳併佼使侃例侍供依侠価侭侮侯侵侶便係促",
            "俄俊俗保信俣修俳俵俸俺倉個倍倒倖候借倣値倦倫倭倶倹偉偏停健偲側偵偶偽傍傑傘備催傭債傷傾僅働像僑僕僚僧僻儀億儒償優儲",
            "允元兄充兆兇先光克免兎児党兜入全八公六共兵其具典兼内円冊再冒冗写冠冥冨冬冴冶冷凄准凋凌凍凝几凡処凧凪凱凶凸凹出函",
            "刀刃分切刈刊刑列初判別利到制刷券刺刻剃則削前剖剛剣剤剥副剰割創劃劇劉",
            "力功加劣助努劫励労効劾勃勅勇勉動勘務勝募勢勤勧勲勺勾勿匁匂包化北匙匝匠匡匪匹区医匿",
            "十千升午半卑卒卓協南単博卜占卦卯印危即却卵卸卿厄厘厚原厨厩厭厳去参",
            "又叉及友双反収叔取受叙叛叡叢口古句叩只叫召可台叱史右叶号司吃各合吉吊吋同名后吏吐向君吟吠否含吸吹吻吾",
            "呂呆呈呉告呑周呪味呼命咋和咲咳咽哀品哉員哨哩哲唄唆唇唐唖唯唱唾啄商問啓善喉喋喚喜喝喧喪喫喬喰営嗣嘆嘉嘗嘘嘩嘱噂噌噛器噴噸噺嚇嚢",
            "囚四回因団困囲図固国圃圏園土圧在圭地坂均坊坐坑坤坦坪垂型垢垣埋城埜域埠埴執培基埼堀堂堅堆堕堤堪堰報場堵堺",
            "塀塁塊塑塔塗塘塙塚塞塩填塵塾境墓増墜墨墳墾壁壇壊壌壕士壬壮声壱売壷変",
            "夏夕外夙多夜夢大天太夫央失夷奄奇奈奉奏契奔套奥奨奪奮女奴好如妃妄妊妓妖妙妥妨妬妹妻妾",
            "姉始姐姑姓委姥姦姪姫姶姻姿威娃娘娠娩娯娼婁婆婚婦婿媒媛嫁嫉嫌嫡嬉嬢嬬嬰",
            "子孔字存孜孝孟季孤学孫宅宇守安宋完宍宏宕宗官宙定宛宜宝実客宣室宥宮宰害宴宵家容宿寂寄寅密富寒寓寛寝察寡寧審寮寵",
            "寸寺対寿封専射将尉尊尋導小少尖尚尤尭就尺尻尼尽尾尿局居屈届屋屍屑展属屠屡層履屯",
            "山岐岡岨岩岬岱岳岸峠峡峨峯峰島峻崇崎崖崩嵐嵩嵯嶋嶺巌川州巡巣工左巧巨差己巳巴巷巻巽巾市布帆希帖帝帥師席帯帰帳常帽幅幌幕幡幣",
            "干平年幸幹幻幼幽幾庁広庄庇床序底庖店庚府度座庫庭庵庶康庸廃廉廊廓廟廠延廷建廻廼廿",
            "弁弄弊式弐弓弔引弗弘弛弟弥弦弧弱張強弼弾彊当形彦彩彪彫彬彰影役彼往征径待律後徐徒従得御復循微徳徴徹徽",
            "心必忌忍志忘忙応忠快念忽怒怖怜思怠急性怨怪怯恋恐恒恕恢恥恨恩恭息恰恵悉悌悔悟悠患悦悩悪悲悶悼情惇惑惚惜惟惣惨惰想惹",
            "愁愈愉意愚愛感慈態慌慎慕慢慣慧慨慮慰慶慾憂憎憐憤憧憩憲憶憾懇懐懲懸",
            "戊戎成我戒或戚戟戦戯戴戸戻房所扇扉手才打払托扮扱扶批承技抄把抑投抗折抜択披抱抵抹押抽担",
            "拍拐拒拓拘拙招拝拠拡括拭拳拶拷拾持指按挑挙挟挨挫振挺挽挿捉捌捕捗捜捧捨据捲捷捺捻掃授掌排掘掛掠採探接控推掩措掬掲掴掻",
            "揃描提揖揚換握揮援揺損搬搭携搾摂摘摩摸摺撃撒撚撞撤撫播撮撰撲撹擁操擢擦擬擾支改攻放政故敏救敗教敢散敦敬数整敵敷",
            "文斉斌斎斐斑斗料斜斡斤斥斧斬断斯新方於施旅旋族旗既",
            "日旦旧旨早旬旭旺昂昆昇昌明昏易昔星映春昧昨昭是昼時晃晋晒晦晩普景晴晶智暁暇暑暖暗暢暦暫暮暴曇曙曜曝曲曳更書曹曽曾替最",
            "月有朋服朔朕朗望朝期木未末本札朱朴机朽杉李杏材村杓杖杜束条杢来杭杯東杵杷松板枇析枕林枚果枝枠枢枯架柁柄柊柏某柑染柔柘柚柱柳柴柵査柾柿",
            "栂栃栄栓栖栗校栢株栴核根格栽桁桂桃案桐桑桓桔桜桝桟桧桶梁梅梓梗梢梧梨梯械梱梶梼棄棉棋棒棚棟森棲棺",
            "椀椅椋植椎椙椛検椴椿楊楓楕楚楠楢業楯楳極楼楽概榊榎榔榛構槌槍様槙槻槽樋樗標樟模権横樫樵樹樺樽橋橘機橡橿檀檎櫓櫛櫨欄欝",
            "欠次欣欧欲欺欽款歌歎歓止正此武歩歪歯歳歴死殆殉殊残殖殴段殺殻殿毅母毎毒比毘毛氏民気",
            "水氷永氾汀汁求汎汐汗汚汝江池汰汲決汽沃沈沌沓沖沙没沢沫河沸油治沼沿況泉泊泌法泡波泣泥注泰泳洋洗洛洞津洩洪洲活派流浄浅浜浦浩浪浬浮浴海浸消",
            "涌涙涛涜涯液涼淀淋淑淘淡淫深淳淵混添清渇済渉渋渓渚減渠渡渥渦温測港湊湖湘湛湧湯湾湿満溌源準溜溝溢溶溺",
            "滅滋滑滝滞滴漁漂漆漉漏演漕漠漢漣漫漬漸潅潔潜潟潤潮潰澄澗澱激濁濃濠濡濫濯瀕瀞瀦瀧瀬灘",
            "火灯灰灸灼災炉炊炎炭点為烈烏烹焔焚無焦然焼煉煎煙煤照煩煮煽熊熔熟熱燃燈燐燕燥燦燭爆爪爵父爺爽爾",
            "片版牌牒牙牛牝牟牡牢牧物牲特牽犀犠犬犯状狂狐狗狙狛狩独狭狸狼狽猛猟猪猫献猶猷猿獄獅獣獲",
            "玄率玉王玖玩玲珂珊珍珠珪班現球理琉琢琳琴琵琶瑚瑛瑞瑠瑳璃璧環璽瓜瓢瓦瓶甑甘甚甜生産甥用甫田由甲申男町画界畏畑畔留畜畝畠畢略畦番異畳畷畿",
            "疋疎疏疑疫疲疹疾病症痔痕痘痛痢痩痴療癌癒癖発登白百的皆皇皐皮皿盃盆盈益盗盛盟監盤目盲直相盾省眉看県真眠眺眼着睡督睦瞥瞬瞭瞳",
            "矛矢知矧矩短矯石砂研砕砥砦砧砲破砺砿硝硫硬硯硲碁碇碍碑碓碕碗碧碩確磁磐磨磯礁礎",
            "示礼社祁祇祈祉祐祖祝神祢祥票祭祷禁禄禅禍禎福禦禰禽禾禿秀私秋科秒秘租秤秦秩称移稀程税稔稗稚稜種稲稼稽稿穀穂穆積穎穏穐穣穫",
            "穴究空穿突窃窄窒窓窟窪窮窯窺竃立竜章竣童竪端競竹竺竿笈笑笛笠笥符第笹筆筈等筋筏筑筒答策箆箇箔箕算管箪箭箱箸節範篇築篠篤篭簡簸簾簿籍",
            "米籾粁粂粉粋粍粒粕粗粘粛粟粥粧精糊糎糖糞糟糠糧",
            "糸系糾紀約紅紋納紐純紗紘紙級紛素紡索紫紬累細紳紹紺終絃組絆経結絞絡絢給統絵絶絹継続綜綬維綱網綴綺綻綾綿緊緋総緑緒線締編緩緬緯練緻縁縄縛縞縦縫縮績繁繊繋繍織繕繭繰纂纏",
            "缶罪罫置罰署罵罷羅羊美群羨義羽翁翌習翠翫翰翻翼耀老考者而耐耕耗耳耶耽聖聞聡聯聴職聾肇",
            "肉肋肌肖肘肝股肢肥肩肪肯肱育肴肺胃胆背胎胞胡胤胴胸能脂脅脆脇脈脊脚脱脳脹腎腐腔腕腫腰腸腹腺腿膏膚膜膝膨膳膿臆臓臣臥臨自臭至致臼興",
            "舌舎舗舘舛舜舞舟航般舵舶舷船艇艦艮良色艶",
            "芋芙芝芥芦芭芯花芳芸芹芽苅苑苓苔苗苛若苦苧苫英茂茄茅茎茜茨茶茸草荊荏荒荘荷荻莞莫莱菅菊菌菓菖菜菟菩華菰菱",
            "萄萌萎萩萱落葉葎著葛葡董葦葬葱葵葺蒋蒐蒔蒙蒜蒲蒸蒼蓄蓉蓋蓑蓬蓮蔀蔑蔓蔚蔦蔭蔵蔽蕃蕉蕊蕎蕗蕨蕩蕪",
            "薄薗薙薦薩薪薫薬薮薯藁藍藤藩藷藻蘇蘭",
            "虎虐虚虜虞虫虹虻蚊蚕蚤蛇蛋蛍蛎蛙蛤蛭蛮蛸蛾蜂蜘蜜蝉蝋蝕蝦蝶蝿融螺蟹蟻",
            "血衆行術街衛衝衡衣表衰衷衿袈袋袖被袴袷裁裂装裏裕補裟裡裳裸製裾複褐褒襖襟襲",
            "西要覆覇見規視覗覚覧親観角解触",
            "言訂計訊討訓託記訟訣訪設許訳訴診註証詐詑詔評詞詠詣試詩詫詮詰話該詳誇誉誌認誓誕誘語誠誤説読誰課誹誼調談請諌諏諒論諜諦諭諮諸諺諾謀謁謂謄謎謙講謝謡謬謹識譜警議譲護讃讐",
            "谷豆豊豚象豪豹貌貝貞負財貢貧貨販貪貫責貯貰貴買貸費貼貿賀賂賃賄資賊賎賑賓賛賜賞賠賢賦質賭購贈贋",
            "赤赦赫走赴起超越趣趨足距跡跨路跳践踊踏蹄蹟蹴躍身躯",
            "車軌軍軒軟転軸軽較載輔輝輩輪輯輸輿轄轍轟轡辛辞辰辱農辺辻込辿迂迄迅迎近返迦迩迫迭述迷追退送逃逆透逐逓途逗這通逝速造逢連逮週進逸逼遁遂遅遇遊運遍過道達違遜遠遡遣遥適遭遮遵遷選遺遼避還邑那邦邪邸郁郊郎郡部郭郵郷都鄭",
            "酉酋酌配酎酒酔酢酪酬酵酷酸醇醍醐醒醗醜醤醸釆采釈里重野量",
            "金釘釜針釣釦釧鈍鎀鈴鈷鉄鉛鉢鉦鉱鉾銀銃銅銑銘銚銭鋒鋤鋪鋭鋲鋳鋸鋼錆錐錘錠錦錨錫錬錯録鍋鍍鍔鍛鍬鍵鍾鎌鎖鎗鎚鎧鎮鏑鏡鐘鐙鐸鑑鑓",
            "長門閃閉開閏閑間関閣閤閥閲闇闘阜阪防阻阿陀附降限陛院陣除陥陪陰陳陵陶陸険陽隅隆隈隊階随隔隙際障隠隣隷隻隼",
            "雀雁雄雅集雇雌雑雛離難雨雪雫雰雲零雷電需震霊霜霞霧露青靖静非面",
            "革靭靴鞄鞍鞘鞠鞭韓韮音韻響",
            "頁頂頃項順須預頑頒頓頗領頚頬頭頴頻頼題額顎顔顕願顛類顧風飛食飢飯飲飴飼飽飾餅養餌餐餓館饗",
            "首香馨馬馳馴駁駄駅駆駈駐駒駕駿騎騒験騨騰驚骨骸髄高髪髭",
            "鬼魁魂魅魔魚魯鮎鮒鮪鮫鮭鮮鯉鯖鯛鯨鯵鰍鰐鰭鰯鰹鰻鱈鱒鱗",
            "鳥鳩鳳鳴鳶鴇鴎鴛鴨鴫鴬鴻鵜鵠鵡鵬鶏鶴鷲鷹鷺鹸鹿麓麗麟麦麹麺麻麿黄黍黒黙黛鼎鼓鼠鼻齢龍",
            // Full-width ASCII variants used in the game font
            "！＃＄％＆（）＊＋，－．／？",
            // Half-width katakana punctuation and voiced-mark combiners
            "｡｢｣､･ﾞﾟ",
            // Full-width yen sign
            "￥",
        );
        for c in jp.chars() {
            set.insert(c);
        }

        set
    })
}

/// Validates a DLC key: only alphanumeric characters allowed.
pub fn dlc_key(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

/// Validates a DLC project field (artist, title, album):
/// only characters included in the fonts the game uses are allowed.
pub fn field(input: &str) -> String {
    let chars = usable_chars();
    input.chars().filter(|c| chars.contains(c)).collect()
}

/// Validates a DLC project sort field: must start with an alphanumeric character.
pub fn sort_field(input: &str) -> String {
    let trimmed = input.trim_start_matches(|c: char| !c.is_ascii_alphanumeric());
    trimmed.to_string()
}

/// Removes English articles (a, an, the) from the beginning of the input string.
pub fn remove_articles(input: &str) -> String {
    let lower = input.to_lowercase();
    if lower.starts_with("the ") {
        input["the ".len()..].to_string()
    } else if lower.starts_with("an ") {
        input["an ".len()..].to_string()
    } else if lower.starts_with("a ") {
        input["a ".len()..].to_string()
    } else {
        input.to_string()
    }
}

/// Removes diacritics from the string using Unicode NFD decomposition.
fn remove_diacritics(input: &str) -> String {
    input
        .nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .nfc()
        .collect()
}

/// Validates a filename without the extension.
/// Removes diacritics, strips characters outside `[^ 0-9a-zA-Z_-]`, and
/// replaces whitespace runs with a single dash.
pub fn file_name(input: &str) -> String {
    let no_diacritics = remove_diacritics(input);

    // Keep only space, alphanumeric, underscore, and hyphen
    let filtered: String = no_diacritics
        .chars()
        .filter(|&c| c == ' ' || c.is_ascii_alphanumeric() || c == '_' || c == '-')
        .collect();

    // Replace runs of spaces with a single dash
    let mut result = String::with_capacity(filtered.len());
    let mut in_space = false;
    for c in filtered.chars() {
        if c == ' ' {
            if !in_space && !result.is_empty() {
                result.push('-');
            }
            in_space = true;
        } else {
            in_space = false;
            result.push(c);
        }
    }
    // Remove any trailing dash introduced by trailing spaces
    result.trim_end_matches('-').to_string()
}
