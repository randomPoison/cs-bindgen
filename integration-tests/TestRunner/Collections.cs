using System.Collections.Generic;
using Xunit;

namespace TestRunner
{
    public class Collections
    {
        [Fact]
        public void ReturnVecInt()
        {
            var expected = IntegrationTests.ReturnVecI32();
            var actual = new List<int>() { 1, 2, 3, 4 };
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void ReturnVecIntManyTimes()
        {
            for (var count = 0; count < 100_000; count += 1)
            {
                ReturnVecInt();
            }
        }

        [Fact]
        public void ReturnVecIntRepeated()
        {
            for (var count = 0; count < 100_000; count += 1)
            {
                ReturnVecInt();
            }
        }

        [Fact]
        public void ReturnVecFloat()
        {
            var expected = IntegrationTests.ReturnVecF32();
            var actual = new List<float>() { 1.0f, 2.1f, 3.123f, 4.00000004f };
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void ReturnVecBool()
        {
            var expected = IntegrationTests.ReturnVecBool();
            var actual = new List<bool>() { true, false, true, true };
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void ReturnStructList()
        {
            var expected = new List<CopyStruct>() {
                new CopyStruct(33),
                new CopyStruct(12345),
            };
            var actual = IntegrationTests.StructVecRoundTrip(expected);
            Assert.Equal(expected, actual);
            Assert.Equal(expected.Count, actual.Count);
            Assert.Equal(expected[0].Bar, actual[0].Bar);
            Assert.Equal(expected[1].Bar, actual[1].Bar);
        }

        [Fact]
        public void ReturnSimpleEnumList()
        {
            var expected = new List<SimpleCEnum>() { SimpleCEnum.Foo, SimpleCEnum.Bar, SimpleCEnum.Baz };
            var actual = IntegrationTests.ReturnSimpleEnumVec();
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void ReturnSimpleEnumListManyTimes()
        {
            for (var count = 0; count < 100_000; count += 1)
            {
                ReturnSimpleEnumList();
            }
        }

        [Fact]
        public void ReturnDataEnumList()
        {
            var expected = new List<IDataEnum>()
            {
                new DataEnum.Foo(),
                new DataEnum.Bar("Cool string"),
                new DataEnum.Coolness(new InnerEnum.Coolest(SimpleCEnum.Foo)),
            };
            var actual = IntegrationTests.ReturnDataEnumVec();
            Assert.Equal(expected, actual);
        }

        [Fact]
        public void DataEnumListManyTimes()
        {
            for (var count = 0; count < 100_000; count += 1)
            {
                ReturnDataEnumList();
            }
        }
    }
}
