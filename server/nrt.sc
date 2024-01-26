(
var testOne = "";
var log_ = Io.log("Running nrt script");
var token = "xsynx";
var writeOut = {|outPath|
	var msg = [token, outPath, token].join;
	msg.postln;
};
var failWithError = {|msg|
    writeOut.value("error");
	msg = writeOut.value("NRTError: " ++ msg);
	1.exit;
};
Io.log("Starting the recording");
try {
	var score = Score.new;
	var def = {|name, ugen|
		SynthDef(name, {|out=0, freq, amp, dur, lpf, hpf|
			var adsr = [[0, 0.05], [0.1, 0.8], [0.2, 1], [0.9, 1], [dur, 0]];
			var env =[Env.perc.ar, Env.triangle.ar, EnvGen.ar(Env.pairs(adsr))].choose;
			var sig = env * ugen.ar(freq:freq, mul: amp/10);
			sig = HPF.ar(LPF.ar(sig, 22000), 20);

			Out.ar(out, sig!2);
			DetectSilence.ar(sig, doneAction:2);
		});
	};
    var synthTypes = ["sine", "saw, 'pulse", "triangle"];
	var isSample = {|part|
        part["sound"]["filepath"].isString && part["osc"].isNil;
    };

	var synths = (
		"sine": SinOsc,
		"saw": Saw,
		"pulse": Pulse,
		"triangle": LFTri
	);
    var outPath  = "";
    var template = nil;
    var ready = {
        var inPath = thisProcess.argv[0];
        var ok = if (inPath.isString.not || inPath.endsWith(".json").not, {
	    Io.log(inPath);
            failWithError.value("Must provide a JSON template for the syntehsizer");
        });

        template = inPath.parseYAMLFile;
		synths.keysValuesDo({|name, ugen|
			synths.put(name, def.value(name, ugen));
		});
		synths["sample"] = SynthDef.new(\sample, {|out, bufnum, freq=300, amp=0.1, dur=4, root=1, hpf=20, lpf=22000|

			var sig = amp * PlayBuf.ar(2, bufnum, BufRateScale.kr(bufnum) * root);
			sig = HPF.ar(LPF.ar(sig, 22000), 20);

			Out.ar( out, sig );
			DetectSilence.ar(sig, doneAction: 2);
		});

        outPath =thisProcess.argv[1];
        ok = if (outPath.isString.not || outPath.endsWith(".aiff").not, {
            failWithError.value("Must provide a JSON template for the syntehsizer");
        });
	}.value;
	var timestamp = 0;
	var root = template["conf"]["root"].asFloat;
	var cps = template["conf"]["cps"].asFloat;
	var durscale = cps.reciprocal;
	var buffStuff = (
		ids: (),
		buffers: [],
		loadMsgs: [],
		afterLoadMsgs: [],
		freeMsgs: []
	);

	var samples = template["parts"]
        .select({|part|
            part["sound"]["filepath"].isString && part["sound"]["osc"].isNil;
        })
        .collect({|part|
            var sample = part["sound"]["filepath"].asAbsolutePath;
            var buffer = Buffer.new;
            // Create a buffer to read the sample and map it directly to this partId for synth argument
            buffStuff[\ids].put(part.hash.asString, buffer.bufnum);
            buffStuff[\buffers].add(buffer);
            buffStuff[\loadMsgs].add([\b_allocRead, buffer.bufnum, sample, 0, -1, nil]);
            buffStuff[\freeMsgs].add(buffer.freeMsg);
            sample;
        });

	var loadingOffset = if (samples.size < 1, 0.1, 1);
	var options, time;
	var onComplete = {
		writeOut.value("completed")
	};

	synths.values.do({|synth|
		score.add([0.0, ['/d_recv', synth.asBytes]]);
	});

	buffStuff[\loadMsgs].do({|msg|
		score = score.add([0, msg]);
	});

	buffStuff[\afterLoadMsgs].do({|msg|
		score = score.add([0 + loadingOffset, msg]);
	});

	template["parts"].do({|part, i|
        var sound = part["sound"];
        var melody = part["motes"];

        melody.do({|motes|
            var dur, freq, amp, minFreq, maxFreq;
            var s;

            time = 0;
            minFreq = sound["minFreq"].asFloat;
            maxFreq = sound["maxFreq"].asFloat;
            motes.do({|mote|
                mote = mote.collect(_.asFloat);
                dur = mote.at(0) * durscale;
                freq = root * mote.at(1);
                amp = mote.at(2);
                amp = 0.1;
				// Io.log(["using filter values", maxFreq.class, minFreq.class, maxFreq, minFreq]);
                if (dur.isPositive, {
                    var synth = if (isSample.value(part), { \sample }, {  sound["osc"].asSymbol });

                    var args = [freq: freq, amp: amp, dur: dur, cps: cps, lpf: maxFreq, hpf: minFreq];
                    if (isSample.value(part), {
                        args = args ++ [\bufnum, buffStuff[\ids][part.hash.asString]];
                    });

                    s = Synth.basicNew(synth);
                    score.add([time, s.newMsg(args: args)]);
                });
                time = time + dur.abs;
            });
        })
	});

	buffStuff[\freeMsgs].do({|msg|
		score = score.add([time + loadingOffset, msg]);
	});

	ScoreWriter.recordAndExit(score, outPath, onComplete);
} {|err|

	Io.log("NRT error");
	Io.log(err.errorString);
	failWithError.value(err.errorString);
}
)


