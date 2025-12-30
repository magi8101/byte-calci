//! Calculator GUI Application
//!
//! egui-based graphical interface showing:
//!   - Input expression
//!   - Tokenized output
//!   - AST visualization
//!   - Bytecode disassembly
//!   - VM execution result
//!   - Memory/GC statistics
//!   - Time-travel debugging with stack visualization

use eframe::egui;
use crate::ast::Expr;
use crate::bytecode::Chunk;
use crate::codegen::CodeGenerator;
use crate::disassembler::Disassembler;
use crate::gc::GcStats;
use crate::memory::MemoryStats;
use crate::parser::{ParseError, Parser};
use crate::tokenizer::{Token, Tokenizer, TokenizerError};
use crate::vm::{ExecutionStep, VirtualMachine, VmError};

/// Compilation pipeline result
#[allow(dead_code)]
struct CompilationResult {
    input: String,
    tokens: Option<Result<Vec<Token>, TokenizerError>>,
    ast: Option<Result<Expr, ParseError>>,
    chunk: Option<Chunk>,
    disassembly: String,
    result: Option<Result<f64, VmError>>,
    execution_trace: Vec<ExecutionStep>,
    /// Memory statistics captured from VM after execution
    memory_stats: Option<MemoryStats>,
    /// GC statistics captured from VM after execution
    gc_stats: Option<GcStats>,
}

impl Default for CompilationResult {
    fn default() -> Self {
        Self {
            input: String::new(),
            tokens: None,
            ast: None,
            chunk: None,
            disassembly: String::new(),
            result: None,
            execution_trace: Vec::new(),
            memory_stats: None,
            gc_stats: None,
        }
    }
}

impl CompilationResult {
    fn compile(input: &str) -> Self {
        let mut result = CompilationResult {
            input: input.to_string(),
            ..Default::default()
        };

        // Tokenize
        let mut tokenizer = Tokenizer::new(input);
        result.tokens = Some(tokenizer.tokenize());

        // Parse
        if let Some(Ok(ref tokens)) = result.tokens {
            let mut parser = Parser::new(tokens.clone());
            result.ast = Some(parser.parse());
        }

        // Compile
        if let Some(Ok(ref ast)) = result.ast {
            let chunk = CodeGenerator::new().compile(ast);
            result.disassembly = Disassembler::format_with_hex(&chunk);
            result.chunk = Some(chunk);
        }

        // Execute
        if let Some(ref chunk) = result.chunk {
            let mut vm = VirtualMachine::new();
            vm.enable_tracing();
            result.result = Some(vm.execute(chunk));
            result.execution_trace = vm.trace().to_vec();
            // Capture stats from the VM before it drops
            result.memory_stats = Some(vm.memory_stats().clone());
            result.gc_stats = Some(vm.gc_stats().clone());
        }

        result
    }
}

/// Calculator application state
pub struct CalculatorApp {
    /// Current input expression
    input: String,
    /// History of calculations
    history: Vec<(String, String)>,
    /// Current compilation result
    compilation: CompilationResult,
    /// Show detailed view
    show_details: bool,
    /// Show execution trace
    show_trace: bool,
    /// Time-travel debugging: current step index
    debug_step: usize,
    /// Whether time-travel debugger is active
    debugger_active: bool,
}

impl Default for CalculatorApp {
    fn default() -> Self {
        Self {
            input: String::new(),
            history: Vec::new(),
            compilation: CompilationResult::default(),
            show_details: true,
            show_trace: false,
            debug_step: 0,
            debugger_active: false,
        }
    }
}

impl CalculatorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn calculate(&mut self) {
        if self.input.trim().is_empty() {
            return;
        }

        self.compilation = CompilationResult::compile(&self.input);
        // Reset debugger to start
        self.debug_step = 0;

        // Add to history
        let result_str = match &self.compilation.result {
            Some(Ok(value)) => format!("{}", value),
            Some(Err(e)) => format!("Error: {}", e),
            None => String::from("No result"),
        };
        self.history.push((self.input.clone(), result_str));
    }

    fn insert_text(&mut self, text: &str) {
        self.input.push_str(text);
    }

    fn clear_input(&mut self) {
        self.input.clear();
        self.compilation = CompilationResult::default();
    }

    fn backspace(&mut self) {
        self.input.pop();
    }
}

impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel with title
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Bytecode Calculator");
                ui.separator();
                ui.checkbox(&mut self.show_details, "Show Details");
                ui.checkbox(&mut self.show_trace, "Show Trace");
                ui.checkbox(&mut self.debugger_active, "Debugger");
            });
        });

        // Left panel with calculator buttons
        egui::SidePanel::left("calculator_panel")
            .min_width(280.0)
            .show(ctx, |ui| {
                self.render_calculator(ui);
            });

        // Central panel with details
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.show_details {
                self.render_details(ui);
            } else {
                self.render_history(ui);
            }
        });
    }
}

impl CalculatorApp {
    fn render_calculator(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Input field
            ui.group(|ui| {
                ui.label("Expression:");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.input)
                        .desired_width(260.0)
                        .font(egui::TextStyle::Monospace),
                );

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.calculate();
                }
            });

            // Result display
            ui.group(|ui| {
                ui.label("Result:");
                let result_text = match &self.compilation.result {
                    Some(Ok(value)) => {
                        if value.fract() == 0.0 && value.abs() < 1e15 {
                            format!("{}", *value as i64)
                        } else {
                            format!("{:.10}", value)
                                .trim_end_matches('0')
                                .trim_end_matches('.')
                                .to_string()
                        }
                    }
                    Some(Err(e)) => format!("{}", e),
                    None => String::new(),
                };
                ui.add(
                    egui::TextEdit::singleline(&mut result_text.as_str())
                        .desired_width(260.0)
                        .font(egui::TextStyle::Monospace),
                );
            });

            ui.add_space(10.0);

            // Calculator buttons
            self.render_buttons(ui);
        });
    }

    fn render_buttons(&mut self, ui: &mut egui::Ui) {
        let button_size = egui::vec2(60.0, 35.0);
        let small_button = egui::vec2(45.0, 30.0);

        // Tab selector for function categories
        ui.horizontal(|ui| {
            ui.label("Functions:");
        });

        // Function buttons row 1 - Trig
        ui.horizontal(|ui| {
            if ui.add_sized(small_button, egui::Button::new("sin")).clicked() {
                self.insert_text("sin(");
            }
            if ui.add_sized(small_button, egui::Button::new("cos")).clicked() {
                self.insert_text("cos(");
            }
            if ui.add_sized(small_button, egui::Button::new("tan")).clicked() {
                self.insert_text("tan(");
            }
            if ui.add_sized(small_button, egui::Button::new("asin")).clicked() {
                self.insert_text("asin(");
            }
            if ui.add_sized(small_button, egui::Button::new("acos")).clicked() {
                self.insert_text("acos(");
            }
        });

        // Function buttons row 2 - Hyperbolic
        ui.horizontal(|ui| {
            if ui.add_sized(small_button, egui::Button::new("sinh")).clicked() {
                self.insert_text("sinh(");
            }
            if ui.add_sized(small_button, egui::Button::new("cosh")).clicked() {
                self.insert_text("cosh(");
            }
            if ui.add_sized(small_button, egui::Button::new("tanh")).clicked() {
                self.insert_text("tanh(");
            }
            if ui.add_sized(small_button, egui::Button::new("atan")).clicked() {
                self.insert_text("atan(");
            }
            if ui.add_sized(small_button, egui::Button::new("exp")).clicked() {
                self.insert_text("exp(");
            }
        });

        // Function buttons row 3 - Math
        ui.horizontal(|ui| {
            if ui.add_sized(small_button, egui::Button::new("sqrt")).clicked() {
                self.insert_text("sqrt(");
            }
            if ui.add_sized(small_button, egui::Button::new("cbrt")).clicked() {
                self.insert_text("cbrt(");
            }
            if ui.add_sized(small_button, egui::Button::new("log")).clicked() {
                self.insert_text("log(");
            }
            if ui.add_sized(small_button, egui::Button::new("ln")).clicked() {
                self.insert_text("ln(");
            }
            if ui.add_sized(small_button, egui::Button::new("log2")).clicked() {
                self.insert_text("log2(");
            }
        });

        // Function buttons row 4 - Rounding
        ui.horizontal(|ui| {
            if ui.add_sized(small_button, egui::Button::new("abs")).clicked() {
                self.insert_text("abs(");
            }
            if ui.add_sized(small_button, egui::Button::new("floor")).clicked() {
                self.insert_text("floor(");
            }
            if ui.add_sized(small_button, egui::Button::new("ceil")).clicked() {
                self.insert_text("ceil(");
            }
            if ui.add_sized(small_button, egui::Button::new("round")).clicked() {
                self.insert_text("round(");
            }
            if ui.add_sized(small_button, egui::Button::new("sign")).clicked() {
                self.insert_text("sign(");
            }
        });

        // Function buttons row 5 - Combinatorics
        ui.horizontal(|ui| {
            if ui.add_sized(small_button, egui::Button::new("n!")).clicked() {
                self.insert_text("!");
            }
            if ui.add_sized(small_button, egui::Button::new("nPr")).clicked() {
                self.insert_text("nPr(");
            }
            if ui.add_sized(small_button, egui::Button::new("nCr")).clicked() {
                self.insert_text("nCr(");
            }
            if ui.add_sized(small_button, egui::Button::new("gcd")).clicked() {
                self.insert_text("gcd(");
            }
            if ui.add_sized(small_button, egui::Button::new("lcm")).clicked() {
                self.insert_text("lcm(");
            }
        });

        // Function buttons row 6 - Arrays
        ui.horizontal(|ui| {
            if ui.add_sized(small_button, egui::Button::new("sum")).clicked() {
                self.insert_text("sum([");
            }
            if ui.add_sized(small_button, egui::Button::new("avg")).clicked() {
                self.insert_text("avg([");
            }
            if ui.add_sized(small_button, egui::Button::new("min")).clicked() {
                self.insert_text("min([");
            }
            if ui.add_sized(small_button, egui::Button::new("max")).clicked() {
                self.insert_text("max([");
            }
            if ui.add_sized(small_button, egui::Button::new("len")).clicked() {
                self.insert_text("len([");
            }
        });

        ui.add_space(5.0);

        // Number pad
        ui.horizontal(|ui| {
            if ui.add_sized(button_size, egui::Button::new("7")).clicked() {
                self.insert_text("7");
            }
            if ui.add_sized(button_size, egui::Button::new("8")).clicked() {
                self.insert_text("8");
            }
            if ui.add_sized(button_size, egui::Button::new("9")).clicked() {
                self.insert_text("9");
            }
            if ui.add_sized(button_size, egui::Button::new("/")).clicked() {
                self.insert_text("/");
            }
        });

        ui.horizontal(|ui| {
            if ui.add_sized(button_size, egui::Button::new("4")).clicked() {
                self.insert_text("4");
            }
            if ui.add_sized(button_size, egui::Button::new("5")).clicked() {
                self.insert_text("5");
            }
            if ui.add_sized(button_size, egui::Button::new("6")).clicked() {
                self.insert_text("6");
            }
            if ui.add_sized(button_size, egui::Button::new("*")).clicked() {
                self.insert_text("*");
            }
        });

        ui.horizontal(|ui| {
            if ui.add_sized(button_size, egui::Button::new("1")).clicked() {
                self.insert_text("1");
            }
            if ui.add_sized(button_size, egui::Button::new("2")).clicked() {
                self.insert_text("2");
            }
            if ui.add_sized(button_size, egui::Button::new("3")).clicked() {
                self.insert_text("3");
            }
            if ui.add_sized(button_size, egui::Button::new("-")).clicked() {
                self.insert_text("-");
            }
        });

        ui.horizontal(|ui| {
            if ui.add_sized(button_size, egui::Button::new("0")).clicked() {
                self.insert_text("0");
            }
            if ui.add_sized(button_size, egui::Button::new(".")).clicked() {
                self.insert_text(".");
            }
            if ui.add_sized(button_size, egui::Button::new("^")).clicked() {
                self.insert_text("^");
            }
            if ui.add_sized(button_size, egui::Button::new("+")).clicked() {
                self.insert_text("+");
            }
        });

        // Constants and operators row
        ui.horizontal(|ui| {
            if ui.add_sized(button_size, egui::Button::new("pi")).clicked() {
                self.insert_text("pi");
            }
            if ui.add_sized(button_size, egui::Button::new("e")).clicked() {
                self.insert_text("e");
            }
            if ui.add_sized(button_size, egui::Button::new("tau")).clicked() {
                self.insert_text("tau");
            }
            if ui.add_sized(button_size, egui::Button::new("%")).clicked() {
                self.insert_text("%");
            }
        });

        ui.add_space(5.0);

        // Control buttons
        ui.horizontal(|ui| {
            if ui.add_sized(button_size, egui::Button::new("(")).clicked() {
                self.insert_text("(");
            }
            if ui.add_sized(button_size, egui::Button::new(")")).clicked() {
                self.insert_text(")");
            }
            if ui.add_sized(button_size, egui::Button::new("[")).clicked() {
                self.insert_text("[");
            }
            if ui.add_sized(button_size, egui::Button::new("]")).clicked() {
                self.insert_text("]");
            }
        });

        ui.horizontal(|ui| {
            if ui.add_sized(button_size, egui::Button::new(",")).clicked() {
                self.insert_text(",");
            }
            if ui.add_sized(button_size, egui::Button::new("DEL")).clicked() {
                self.backspace();
            }
            if ui.add_sized(button_size, egui::Button::new("CLR")).clicked() {
                self.clear_input();
            }
        });

        ui.add_space(5.0);

        // Calculate button
        if ui
            .add_sized(egui::vec2(260.0, 50.0), egui::Button::new("= Calculate"))
            .clicked()
        {
            self.calculate();
        }
    }

    fn render_details(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Tokens
            ui.collapsing("Tokens", |ui| {
                match &self.compilation.tokens {
                    Some(Ok(tokens)) => {
                        ui.horizontal_wrapped(|ui| {
                            for token in tokens {
                                ui.label(
                                    egui::RichText::new(format!("{}", token))
                                        .monospace()
                                        .background_color(egui::Color32::from_gray(40)),
                                );
                            }
                        });
                    }
                    Some(Err(e)) => {
                        ui.colored_label(egui::Color32::RED, format!("{}", e));
                    }
                    None => {
                        ui.label("No tokens");
                    }
                }
            });

            ui.add_space(5.0);

            // AST
            ui.collapsing("Abstract Syntax Tree", |ui| {
                match &self.compilation.ast {
                    Some(Ok(ast)) => {
                        ui.label(egui::RichText::new(format!("{}", ast)).monospace());
                    }
                    Some(Err(e)) => {
                        ui.colored_label(egui::Color32::RED, format!("{}", e));
                    }
                    None => {
                        ui.label("No AST");
                    }
                }
            });

            ui.add_space(5.0);

            // Bytecode
            ui.collapsing("Bytecode Disassembly", |ui| {
                if !self.compilation.disassembly.is_empty() {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.compilation.disassembly.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY),
                    );
                } else {
                    ui.label("No bytecode generated");
                }
            });

            ui.add_space(5.0);

            // Execution trace
            if self.show_trace {
                ui.collapsing("Execution Trace", |ui| {
                    if self.compilation.execution_trace.is_empty() {
                        ui.label("No trace available");
                    } else {
                        egui::Grid::new("trace_grid")
                            .num_columns(4)
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("IP").strong());
                                ui.label(egui::RichText::new("Opcode").strong());
                                ui.label(egui::RichText::new("Stack Before").strong());
                                ui.label(egui::RichText::new("Stack After").strong());
                                ui.end_row();

                                for step in &self.compilation.execution_trace {
                                    ui.label(format!("0x{:02X}", step.ip));
                                    let op_text = match step.operand {
                                        Some(v) => format!("{} {}", step.opcode, v),
                                        None => format!("{}", step.opcode),
                                    };
                                    ui.label(op_text);
                                    ui.label(format!("{:?}", step.stack_before));
                                    ui.label(format!("{:?}", step.stack_after));
                                    ui.end_row();
                                }
                            });
                    }
                });
            }

            ui.add_space(5.0);

            // Time-travel debugger
            if self.debugger_active && !self.compilation.execution_trace.is_empty() {
                ui.collapsing("Time-Travel Debugger", |ui| {
                    let trace_len = self.compilation.execution_trace.len();
                    
                    ui.horizontal(|ui| {
                        ui.label("Step:");
                        ui.add(
                            egui::Slider::new(&mut self.debug_step, 0..=(trace_len.saturating_sub(1)))
                                .show_value(true)
                                .text(format!("/ {}", trace_len.saturating_sub(1))),
                        );
                    });

                    ui.horizontal(|ui| {
                        if ui.button("|<").clicked() {
                            self.debug_step = 0;
                        }
                        if ui.button("<").clicked() && self.debug_step > 0 {
                            self.debug_step -= 1;
                        }
                        if ui.button(">").clicked() && self.debug_step < trace_len.saturating_sub(1) {
                            self.debug_step += 1;
                        }
                        if ui.button(">|").clicked() {
                            self.debug_step = trace_len.saturating_sub(1);
                        }
                    });

                    ui.separator();

                    if let Some(step) = self.compilation.execution_trace.get(self.debug_step) {
                        // Current instruction
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Instruction:").strong());
                            let op_text = match step.operand {
                                Some(v) => format!("{} {}", step.opcode, v),
                                None => format!("{}", step.opcode),
                            };
                            ui.label(
                                egui::RichText::new(format!("0x{:02X}: {}", step.ip, op_text))
                                    .monospace()
                                    .color(egui::Color32::YELLOW),
                            );
                        });

                        ui.add_space(5.0);

                        // Stack visualization
                        ui.label(egui::RichText::new("Stack State:").strong());
                        
                        ui.horizontal(|ui| {
                            // Stack before
                            ui.vertical(|ui| {
                                ui.label("Before:");
                                self.render_stack_visual(ui, &step.stack_before);
                            });

                            ui.separator();

                            // Stack after
                            ui.vertical(|ui| {
                                ui.label("After:");
                                self.render_stack_visual(ui, &step.stack_after);
                            });
                        });
                    }
                });
            }

            ui.add_space(5.0);

            // Memory stats
            ui.collapsing("Memory Statistics", |ui| {
                if let (Some(mem_stats), Some(gc_stats)) = 
                    (&self.compilation.memory_stats, &self.compilation.gc_stats) 
                {
                    egui::Grid::new("mem_stats_grid")
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Total Allocated:");
                            ui.label(format!("{} bytes", mem_stats.total_allocated));
                            ui.end_row();

                            ui.label("Current Usage:");
                            ui.label(format!("{} bytes", mem_stats.current_usage));
                            ui.end_row();

                            ui.label("Peak Usage:");
                            ui.label(format!("{} bytes", mem_stats.peak_usage));
                            ui.end_row();

                            ui.label("Allocations:");
                            ui.label(format!("{}", mem_stats.allocation_count));
                            ui.end_row();

                            ui.label("GC Collections:");
                            ui.label(format!("{}", gc_stats.collections));
                            ui.end_row();

                            ui.label("Objects Freed:");
                            ui.label(format!("{}", gc_stats.total_objects_freed));
                            ui.end_row();
                        });
                } else {
                    ui.label("No statistics available - run a calculation first");
                }
            });
        });
    }

    /// Render a visual stack representation
    fn render_stack_visual(&self, ui: &mut egui::Ui, stack: &[f64]) {
        if stack.is_empty() {
            ui.label(
                egui::RichText::new("[empty]")
                    .monospace()
                    .color(egui::Color32::GRAY),
            );
            return;
        }

        ui.vertical(|ui| {
            // Show stack top to bottom (reversed)
            for (i, value) in stack.iter().rev().enumerate() {
                let is_top = i == 0;
                let formatted = if value.fract() == 0.0 && value.abs() < 1e10 {
                    format!("{}", *value as i64)
                } else {
                    format!("{:.6}", value)
                };
                
                let text = egui::RichText::new(format!("[{}]", formatted))
                    .monospace();
                
                let text = if is_top {
                    text.color(egui::Color32::LIGHT_GREEN).strong()
                } else {
                    text.color(egui::Color32::LIGHT_GRAY)
                };
                
                ui.label(text);
            }
        });
    }

    fn render_history(&mut self, ui: &mut egui::Ui) {
        ui.heading("Calculation History");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (expr, result) in self.history.iter().rev() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(expr).monospace());
                    ui.label("=");
                    ui.label(egui::RichText::new(result).monospace().strong());
                });
                ui.separator();
            }
        });

        if self.history.is_empty() {
            ui.label("No calculations yet");
        }
    }
}
