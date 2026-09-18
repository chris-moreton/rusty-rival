import importlib.util,json,os,sys,tempfile,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('build_pgo',str(Path(__file__).with_name('build_pgo.py')));build=importlib.util.module_from_spec(spec);spec.loader.exec_module(build)

class BuilderTests(unittest.TestCase):
 def test_frozen_training_has_no_holdout_or_bench_leakage(self):
  data=json.loads((Path(__file__).parent/'pgo/positions.json').read_text())
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'positions.json';p.write_text(json.dumps(data));self.assertEqual(len(build.load_training(p)),80)
   data['heldout'].append(data['training'][0]);p.write_text(json.dumps(data))
   with self.assertRaisesRegex(ValueError,'overlap'):build.load_training(p)
 def test_duplicate_training_rejected(self):
  data=json.loads((Path(__file__).parent/'pgo/positions.json').read_text());data['training'][1]=data['training'][0]
  with tempfile.TemporaryDirectory() as tmp:
   p=Path(tmp)/'positions.json';p.write_text(json.dumps(data))
   with self.assertRaisesRegex(ValueError,'distinct'):build.load_training(p)

 def test_profile_mismatch_and_missing_hot_function_are_rejected(self):
  for line in ['warning: function control flow change detected (hash mismatch) foo', 'warning: no profile data available for function _RNvCxyz11rusty_rival6search6search', 'warning: no profile data available for function _RNvCxyz11rusty_rival10cold_setup']:
   with self.subTest(line=line),self.assertRaises(RuntimeError):build.validate_profile_warnings(build.profile_warnings(line))
  self.assertEqual(build.profile_warnings('warning: no profile data available for function cold_setup')['missing_function_count'],1)
  self.assertEqual(build.profile_warnings('warning: no profile data available for function _RNvCxyz5regex6search')['engine_missing_function_lines'],[])
  self.assertEqual(build.profile_warnings('Finished release profile')['missing_function_count'],0)

 def test_hot_profile_requires_each_direct_function_with_positive_counts(self):
  text=''.join('  _RNvNtCtest11rusty_rival'+name+':\n    Hash: 0x123\n    Counters: 2\n    Block counts: [0, 10]\n' for name in ['6search6search','7quiesce7quiesce','4nnue23update_accumulator_from'])
  self.assertEqual(len(build.audit_hot_counts(text)),3)
  with self.assertRaisesRegex(RuntimeError,'No positive'):build.audit_hot_counts('')
  text=text.replace('23update_accumulator_from:', '23update_accumulator_from_missing:')
  with self.assertRaisesRegex(RuntimeError,'23update_accumulator_from'):build.audit_hot_counts(text)

 def test_current_source_bench_overlap_is_rejected(self):
  data=json.loads((Path(__file__).parent/'pgo/positions.json').read_text())
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp);(root/'src').mkdir();p=root/'positions.json';p.write_text(json.dumps(data))
   (root/'src/uci_bench.rs').write_text('const BENCH_FENS: [&str; 1] = ["'+data['training'][0]['fen']+'"];')
   with self.assertRaisesRegex(ValueError,'CURRENT source bench'):build.load_training(p,root)
 def test_runner_must_support_avx2_and_f16c(self):
  flags='avx avx2 bmi1 bmi2 fma movbe f16c xsave pni ssse3 sse4_1 sse4_2 popcnt cx16 lahf_lm abm'
  self.assertEqual(build.check_runner_isa('flags : '+flags)['logical_cpus_checked'],1)
  for flag in ['avx2','f16c','abm']:
   with self.subTest(flag=flag),self.assertRaisesRegex(RuntimeError,'lacks'):
    build.check_runner_isa('flags : '+' '.join(x for x in flags.split() if x!=flag))
 def fake(self,root,mode):
  p=root/'engine';p.write_text('#!'+sys.executable+'\n'+f'''import sys
from pathlib import Path
for line in sys.stdin:
 c=line.strip()
 if c=='uci':print('id name Rusty Rival {'wrong' if mode=='version' else '1.0.68'}\\nuciok',flush=True)
 elif c=='isready':print('readyok',flush=True)
 elif c.startswith('go'):
  print({'Error: rejected go' if mode=='error' else 'info depth 4 score mate 1 nodes 42 time 1 pv e2e4'+chr(10)+'bestmove e2e4'!r},flush=True)
 elif c=='quit':Path({str(root/'quit')!r}).write_text('yes');break
''');p.chmod(0o755);return p
 def test_short_mating_training_is_recorded_and_engine_quits(self):
  with tempfile.TemporaryDirectory() as tmp:
   r=Path(tmp);p=self.fake(r,'ok');rows=build.train(p,[{'fen':'8/8/8/8/8/8/4P3/K6k w - - 0 1'}],r,'1.0.68',r/'record.json')
   self.assertEqual(rows[0]['reported_nodes'],42);self.assertTrue((r/'quit').exists())
 def test_wrong_version_and_protocol_error_close_engine(self):
  for mode,pattern in [('version','wrong UCI version'),('error','rejected training command')]:
   with self.subTest(mode=mode),tempfile.TemporaryDirectory() as tmp:
    r=Path(tmp);p=self.fake(r,mode)
    with self.assertRaisesRegex(RuntimeError,pattern):build.train(p,[{'fen':'8/8/8/8/8/8/4P3/K6k w - - 0 1'}],r,'1.0.68',r/'record.json')
    self.assertTrue((r/'quit').exists())
if __name__=='__main__':unittest.main()
