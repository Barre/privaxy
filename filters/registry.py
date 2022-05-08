from enum import Enum
import functools
from requests.adapters import HTTPAdapter
import requests
import time
import hashlib
from typing import List, Text

HTTP_MAX_RETRIES = 5


class FilterUrl:
    def __init__(self, filter_url: Text):
        self.filter_url = filter_url

    def url(self) -> Text:
        return self.filter_url

    def hash(self) -> Text:
        return hashlib.sha256(self.filter_url.encode()).hexdigest()


class FilterException(Exception):
    pass


class FilterFetchException(FilterException, requests.exceptions.RequestException):
    pass


class FilterFetchStatusNotOkException(FilterException):
    pass


class FilterGroup(Enum):
    DEFAULT = "default"
    REGIONAL = "regional"
    ADS = "ads"
    PRIVACY = "privacy"
    MALWARE = "malware"
    SOCIAL = "social"


class Filter:
    def __init__(
        self,
        filter_group: FilterGroup,
        url: FilterUrl,
        title: Text,
        enabled_by_default=False,
    ) -> None:
        self.filter_group = filter_group
        self.url = url
        self.title = title
        self.enabled_by_default = enabled_by_default

    def to_dict(self) -> Text:
        return {
            "file_name": f"{self.url.hash()}.txt",
            "title": self.title,
            "group": str(self.filter_group.value),
            "enabled_by_default": self.enabled_by_default,
        }

    def _download(self) -> Text:
        session = requests.Session()

        session.mount("http://", HTTPAdapter(max_retries=HTTP_MAX_RETRIES))
        session.mount("https://", HTTPAdapter(max_retries=HTTP_MAX_RETRIES))

        try:
            response = session.get(f"{self.url.url()}?t={int(time.time())}")
        except requests.exceptions.RequestException as e:
            raise FilterFetchException(e)

        if not response.ok:
            raise FilterFetchStatusNotOkException

        return response.text

    def save_to_registry(self) -> None:
        filter = self._download()

        try:
            with open(f"registry/{self.url.hash()}.txt", "r") as f:
                current_filter = f.read()
        except FileNotFoundError:
            current_filter = ""

        # We strip comments before comparing as some lists
        # are just adding the current timestamp in filter header's comments.
        if _strip_comments_from_filter_list(filter) == _strip_comments_from_filter_list(
            current_filter
        ):
            return

        with open(f"registry/{self.url.hash()}.txt", "w") as f:
            f.write(filter)


def _strip_comments_from_filter_list(filter_list: Text) -> Text:
    filter_list_new = filter_list.splitlines()

    try:
        if filter_list_new[0].startswith("[") and filter_list_new[0].endswith("]"):
            del filter_list_new[0]

    except IndexError:
        return ""

    filter_list_new = [
        filter
        for filter in filter_list_new
        if not filter.startswith("!") and not filter == ""
    ]

    filter_list_new.sort()

    return "\n".join(filter_list_new)


@functools.lru_cache()
def get_filters() -> List[Filter]:
    """
    A filter set mostly derived from https://github.com/gorhill/uBlock/blob/master/assets/assets.json
    """
    return [
        Filter(
            filter_group=FilterGroup.DEFAULT,
            url=FilterUrl(
                "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/filters.txt"
            ),
            title="uBlock filters",
            enabled_by_default=True,
        ),
        Filter(
            filter_group=FilterGroup.DEFAULT,
            url=FilterUrl(
                "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/badware.txt"
            ),
            title="uBlock filters - Badware risks",
            enabled_by_default=True,
        ),
        Filter(
            filter_group=FilterGroup.DEFAULT,
            url=FilterUrl(
                "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/privacy.txt"
            ),
            title="uBlock filters - Privacy",
            enabled_by_default=True,
        ),
        Filter(
            filter_group=FilterGroup.DEFAULT,
            url=FilterUrl(
                "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/resource-abuse.txt"
            ),
            title="uBlock filters - Resource abuse",
            enabled_by_default=True,
        ),
        Filter(
            filter_group=FilterGroup.DEFAULT,
            url=FilterUrl(
                "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/unbreak.txt"
            ),
            title="uBlock filters - Unbreak",
            enabled_by_default=True,
        ),
        Filter(
            filter_group=FilterGroup.ADS,
            url=FilterUrl(
                "https://filters.adtidy.org/extension/ublock/filters/2_without_easylist.txt"
            ),
            title="AdGuard Base",
        ),
        Filter(
            filter_group=FilterGroup.ADS,
            url=FilterUrl("https://filters.adtidy.org/extension/ublock/filters/11.txt"),
            title="AdGuard Mobile Ads",
        ),
        Filter(
            filter_group=FilterGroup.ADS,
            url=FilterUrl("https://easylist.to/easylist/easylist.txt"),
            title="EasyList",
            enabled_by_default=True,
        ),
        Filter(
            filter_group=FilterGroup.PRIVACY,
            url=FilterUrl("https://filters.adtidy.org/extension/ublock/filters/3.txt"),
            title="AdGuard Tracking Protection",
        ),
        Filter(
            filter_group=FilterGroup.PRIVACY,
            url=FilterUrl("https://filters.adtidy.org/extension/ublock/filters/17.txt"),
            title="AdGuard URL Tracking Protection",
        ),
        Filter(
            filter_group=FilterGroup.PRIVACY,
            url=FilterUrl(
                "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/lan-block.txt"
            ),
            title="Block Outsider Intrusion into LAN",
        ),
        Filter(
            filter_group=FilterGroup.PRIVACY,
            url=FilterUrl("https://easylist.to/easylist/easyprivacy.txt"),
            title="EasyPrivacy",
            enabled_by_default=True,
        ),
        Filter(
            filter_group=FilterGroup.MALWARE,
            url=FilterUrl(
                "https://curben.gitlab.io/malware-filter/phishing-filter.txt"
            ),
            title="Phishing URL Blocklist",
        ),
        Filter(
            filter_group=FilterGroup.MALWARE,
            url=FilterUrl("https://curben.gitlab.io/malware-filter/pup-filter.txt"),
            title="PUP Domains Blocklist",
        ),
        Filter(
            filter_group=FilterGroup.SOCIAL,
            url=FilterUrl("https://filters.adtidy.org/extension/ublock/filters/14.txt"),
            title="AdGuard Annoyances",
        ),
        Filter(
            filter_group=FilterGroup.SOCIAL,
            url=FilterUrl("https://filters.adtidy.org/extension/ublock/filters/4.txt"),
            title="AdGuard Social Media",
        ),
        Filter(
            filter_group=FilterGroup.SOCIAL,
            url=FilterUrl("https://secure.fanboy.co.nz/fanboy-antifacebook.txt"),
            title="Anti-Facebook",
        ),
        Filter(
            filter_group=FilterGroup.SOCIAL,
            url=FilterUrl("https://secure.fanboy.co.nz/fanboy-annoyance.txt"),
            title="Fanboy's Annoyance",
        ),
        Filter(
            filter_group=FilterGroup.SOCIAL,
            url=FilterUrl("https://secure.fanboy.co.nz/fanboy-cookiemonster.txt"),
            title="EasyList Cookie",
        ),
        Filter(
            filter_group=FilterGroup.SOCIAL,
            url=FilterUrl("https://easylist.to/easylist/fanboy-social.txt"),
            title="Fanboy's Social",
        ),
        Filter(
            filter_group=FilterGroup.SOCIAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/annoyances.txt"
            ),
            title="uBlock filters - Annoyances",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl("https://easylist-downloads.adblockplus.org/Liste_AR.txt"),
            title="ara: Liste AR",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl("https://stanev.org/abp/adblock_bg.txt"),
            title="BGR: Bulgarian Adblock list",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://filters.adtidy.org/extension/ublock/filters/224.txt"
            ),
            title="CHN: AdGuard Chinese (中文)",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/tomasko126/easylistczechandslovak/master/filters.txt"
            ),
            title="CZE, SVK: EasyList Czech and Slovak",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl("https://easylist.to/easylistgermany/easylistgermany.txt"),
            title="DEU: EasyList Germany",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl("https://adblock.ee/list.php"),
            title="EST: Eesti saitidele kohandatud filter",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/finnish-easylist-addition/finnish-easylist-addition/master/Finland_adb.txt"
            ),
            title="FIN: Adblock List for Finland",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl("https://filters.adtidy.org/extension/ublock/filters/16.txt"),
            title="FRA: AdGuard Français",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl("https://www.void.gr/kargig/void-gr-filters.txt"),
            title="GRC: Greek AdBlock Filter",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/hufilter/hufilter/master/hufilter-ublock.txt"
            ),
            title="HUN: hufilter",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/ABPindo/indonesianadblockrules/master/subscriptions/abpindo.txt"
            ),
            title="IDN, MYS: ABPindo",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/farrokhi/adblock-iran/master/filter.txt"
            ),
            title="IRN: Adblock-Iran",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl("https://adblock.gardar.net/is.abp.txt"),
            title="ISL: Icelandic ABP List",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/easylist/EasyListHebrew/master/EasyListHebrew.txt"
            ),
            title="ISR: EasyList Hebrew",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://easylist-downloads.adblockplus.org/easylistitaly.txt"
            ),
            title="ITA: EasyList Italy",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/gioxx/xfiles/master/filtri.txt"
            ),
            title="ITA: ABP X Files",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl("https://filters.adtidy.org/extension/ublock/filters/7.txt"),
            title="JPN: AdGuard Japanese",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/yous/YousList/master/youslist.txt"
            ),
            title="KOR: YousList",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/EasyList-Lithuania/easylist_lithuania/master/easylistlithuania.txt"
            ),
            title="LTU: EasyList Lithuania",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://notabug.org/latvian-list/adblock-latvian/raw/master/lists/latvian-list.txt"
            ),
            title="LVA: Latvian List",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://easylist-downloads.adblockplus.org/easylistdutch.txt"
            ),
            title="NLD: EasyList Dutch",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/DandelionSprout/adfilt/master/NorwegianList.txt"
            ),
            title="NOR, DNK, ISL: Dandelion Sprouts nordiske filtre",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/MajkiIT/polish-ads-filter/master/polish-adblock-filters/adblock.txt"
            ),
            title="POL: Oficjalne Polskie Filtry do AdBlocka, uBlocka Origin i AdGuarda",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/olegwukr/polish-privacy-filters/master/anti-adblock.txt"
            ),
            title="POL: Oficjalne polskie filtry przeciwko alertom o Adblocku",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl("https://road.adblock.ro/lista.txt"),
            title="ROU: Romanian Ad (ROad) Block List Light",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://easylist-downloads.adblockplus.org/advblock+cssfixes.txt"
            ),
            title="RUS: RU AdList",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://easylist-downloads.adblockplus.org/easylistspanish.txt"
            ),
            title="spa: EasyList Spanish",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl("https://filters.adtidy.org/extension/ublock/filters/9.txt"),
            title="spa, por: AdGuard Spanish/Portuguese",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/betterwebleon/slovenian-list/master/filters.txt"
            ),
            title="SVN: Slovenian List",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/lassekongo83/Frellwits-filter-lists/master/Frellwits-Swedish-Filter.txt"
            ),
            title="SWE: Frellwit's Swedish Filter",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/easylist-thailand/easylist-thailand/master/subscription/easylist-thailand.txt"
            ),
            title="THA: EasyList Thailand",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl("https://filters.adtidy.org/extension/ublock/filters/13.txt"),
            title="TUR: AdGuard Turkish",
        ),
        Filter(
            filter_group=FilterGroup.REGIONAL,
            url=FilterUrl(
                "https://raw.githubusercontent.com/abpvn/abpvn/master/filter/abpvn_ublock.txt"
            ),
            title="VIE: ABPVN List",
        ),
    ]
