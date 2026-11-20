slint::slint! {
    #[edition = "1.19"]
    import { DitherTabs, DitherTabsVariant, DitherMatrix, DitherBloom, DitherPalette, DitherTheme, DitherButton, DitherButtonVariant, DitherButtonSize } from "ui/lib.slint";
    export { DitherEngine } from "ui/lib.slint";

    export component Demo inherits Window {
        title: "DitherTabs";
        background: DitherTheme.background;
        preferred-width: 640px;
        preferred-height: 720px;

        in-out property <DitherTabsVariant> active-variant: DitherTabsVariant.pill;
        in-out property <color> active-color: DitherPalette.blue;
        in-out property <DitherMatrix> active-matrix: DitherMatrix.bayer4;
        in-out property <DitherBloom> active-bloom: DitherBloom.off;
        in-out property <length> active-cell: 2px;

        VerticalLayout {
            padding: 32px;
            spacing: 24px;
            alignment: center;

            // Main Preview Card
            Rectangle {
                background: #141312;
                border-radius: 16px;
                border-width: 1px;
                border-color: #272522;
                clip: true;

                VerticalLayout {
                    padding: 32px;
                    spacing: 28px;
                    alignment: center;

                    // Header toolbar
                    HorizontalLayout {
                        alignment: space-between;
                        Text {
                            text: "Preview";
                            font-size: 13px;
                            font-weight: 500;
                            color: DitherTheme.foreground;
                            vertical-alignment: center;
                        }
                        HorizontalLayout {
                            spacing: 16px;
                            Text { text: "Controls"; font-size: 13px; color: #8a857d; vertical-alignment: center; }
                            Text { text: "Replay"; font-size: 13px; color: #8a857d; vertical-alignment: center; }
                        }
                    }

                    // The DitherTabs component
                    tabs := DitherTabs {
                        tabs: ["Overview", "Deploys", "Settings"];
                        variant: root.active-variant;
                        color: root.active-color;
                        matrix: root.active-matrix;
                        bloom: root.active-bloom;
                        cell: root.active-cell;
                    }

                    // Content panel corresponding to active tab
                    VerticalLayout {
                        spacing: 8px;
                        alignment: center;

                        if tabs.current == 0: VerticalLayout {
                            alignment: center;
                            spacing: 8px;
                            Text {
                                text: "99.98% uptime";
                                font-size: 26px;
                                font-weight: 600;
                                color: DitherTheme.foreground;
                                horizontal-alignment: center;
                            }
                            Text {
                                text: "Production is on commit 4f2a9c1, built in 38 seconds and serving from 14 regions.";
                                font-size: 14px;
                                color: #8a857d;
                                horizontal-alignment: center;
                            }
                        }

                        if tabs.current == 1: VerticalLayout {
                            alignment: center;
                            spacing: 8px;
                            Text {
                                text: "3 today";
                                font-size: 26px;
                                font-weight: 600;
                                color: DitherTheme.foreground;
                                horizontal-alignment: center;
                            }
                            Text {
                                text: "Three deploys today. The last preview went out 12 minutes ago from the fix/cache branch.";
                                font-size: 14px;
                                color: #8a857d;
                                horizontal-alignment: center;
                            }
                        }

                        if tabs.current == 2: VerticalLayout {
                            alignment: center;
                            spacing: 8px;
                            Text {
                                text: "Node 22";
                                font-size: 26px;
                                font-weight: 600;
                                color: DitherTheme.foreground;
                                horizontal-alignment: center;
                            }
                            Text {
                                text: "Builds run on Node 22 with a 2 GB cache. Previews expire after 30 days.";
                                font-size: 14px;
                                color: #8a857d;
                                horizontal-alignment: center;
                            }
                        }
                    }
                }
            }

            // Controls Panel matching the reference
            Rectangle {
                background: #141312;
                border-radius: 12px;
                border-width: 1px;
                border-color: #272522;

                VerticalLayout {
                    padding: 16px;
                    spacing: 16px;

                    Text {
                        text: "CONTROLS";
                        font-size: 11px;
                        font-weight: 600;
                        color: #635f59;
                        letter-spacing: 1px;
                    }

                    HorizontalLayout {
                        spacing: 24px;
                        alignment: space-between;

                        // Variant Control
                        VerticalLayout {
                            spacing: 6px;
                            Text { text: "VARIANT"; font-size: 11px; color: #8a857d; }
                            HorizontalLayout {
                                spacing: 6px;
                                DitherButton {
                                    text: "pill";
                                    size: DitherButtonSize.sm;
                                    variant: root.active-variant == DitherTabsVariant.pill ? DitherButtonVariant.filled : DitherButtonVariant.outline;
                                    color: root.active-variant == DitherTabsVariant.pill ? DitherPalette.blue : #808080;
                                    clicked => { root.active-variant = DitherTabsVariant.pill; }
                                }
                                DitherButton {
                                    text: "underline";
                                    size: DitherButtonSize.sm;
                                    variant: root.active-variant == DitherTabsVariant.underline ? DitherButtonVariant.filled : DitherButtonVariant.outline;
                                    color: root.active-variant == DitherTabsVariant.underline ? DitherPalette.blue : #808080;
                                    clicked => { root.active-variant = DitherTabsVariant.underline; }
                                }
                            }
                        }

                        // Bloom Control
                        VerticalLayout {
                            spacing: 6px;
                            Text { text: "BLOOM"; font-size: 11px; color: #8a857d; }
                            HorizontalLayout {
                                spacing: 6px;
                                DitherButton {
                                    text: "off";
                                    size: DitherButtonSize.sm;
                                    variant: root.active-bloom == DitherBloom.off ? DitherButtonVariant.filled : DitherButtonVariant.outline;
                                    clicked => { root.active-bloom = DitherBloom.off; }
                                }
                                DitherButton {
                                    text: "low";
                                    size: DitherButtonSize.sm;
                                    variant: root.active-bloom == DitherBloom.low ? DitherButtonVariant.filled : DitherButtonVariant.outline;
                                    clicked => { root.active-bloom = DitherBloom.low; }
                                }
                                DitherButton {
                                    text: "high";
                                    size: DitherButtonSize.sm;
                                    variant: root.active-bloom == DitherBloom.high ? DitherButtonVariant.filled : DitherButtonVariant.outline;
                                    clicked => { root.active-bloom = DitherBloom.high; }
                                }
                            }
                        }
                    }

                    // Colour Row
                    VerticalLayout {
                        spacing: 6px;
                        Text { text: "COLOUR"; font-size: 11px; color: #8a857d; }
                        HorizontalLayout {
                            spacing: 8px;
                            for c[i] in [
                                DitherPalette.blue,
                                #0593e6, // cyan/blue
                                DitherPalette.purple,
                                DitherPalette.green,
                                DitherPalette.pink,
                                DitherPalette.orange,
                                DitherPalette.red,
                                DitherPalette.grey
                            ]: Rectangle {
                                width: 22px;
                                height: 22px;
                                border-radius: 11px;
                                background: c;
                                border-width: root.active-color == c ? 2px : 0px;
                                border-color: white;

                                TouchArea {
                                    clicked => { root.active-color = c; }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let demo = Demo::new()?;
    dither_slint::install!(demo);
    demo.run()
}
