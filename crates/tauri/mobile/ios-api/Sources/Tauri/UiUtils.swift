import UIKit

public class UIUtils {
    public static func centerPopover(rootViewController: UIViewController?, popoverController: UIViewController) {
        if let viewController = rootViewController {
            popoverController.popoverPresentationController?.sourceRect = CGRect(x: viewController.view.center.x, y: viewController.view.center.y, width: 0, height: 0)
            popoverController.popoverPresentationController?.sourceView = viewController.view
            popoverController.popoverPresentationController?.permittedArrowDirections = UIPopoverArrowDirection.up
        }
    }
}
