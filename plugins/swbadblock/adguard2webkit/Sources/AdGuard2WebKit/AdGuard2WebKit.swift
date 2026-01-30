// Convert AdGuard filter lines into WebKit-style JSON

import ContentBlockerConverter
import Foundation

/// Core conversion library. Convert a list of lines into valid JSON
func adGuardRulesToWebKitJSON(rules: [String]) -> String {
    let conversionResult = ContentBlockerConverter().convertArray(rules: rules)
    if conversionResult.safariRulesCount == 0 {
        return "[\n]\n"
    }
    return conversionResult.safariRulesJSON
}

/// Convert a C string of AdGuard rules into a C string JSON for the C API using agRules2wkJSON fn
@_cdecl("adguard2webkit__convert")
public func adguard2webkit__convert(
    _ rulesPtr: UnsafePointer<CChar>?
) -> UnsafeMutablePointer<CChar>? {
    guard let rulesPtr = rulesPtr else {
        return nil
    }
    let rulesString = String(cString: rulesPtr)
    let rules = rulesString.components(separatedBy: .newlines)
        .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
        .filter { !$0.isEmpty }
    let jsonResult = adGuardRulesToWebKitJSON(rules: rules)
    return strdup(jsonResult)
}
